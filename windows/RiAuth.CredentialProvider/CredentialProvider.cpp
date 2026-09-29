// Interactive local-account credential provider for the riAuth device protocol.
// The Windows password never goes to DeviceHost. Every serialization first asks
// DeviceHost to redeem a fresh server ticket for the enrolled identity and SID.

#include <windows.h>
#include <credentialprovider.h>
#include <knownfolders.h>
#include <lm.h>
#include <ntsecapi.h>
#include <sddl.h>
#ifndef SECURITY_WIN32
#define SECURITY_WIN32
#endif
#include <security.h>
#include <shlobj.h>
#include <wincred.h>

#include <atomic>
#include <climits>
#include <cstdint>
#include <cstring>
#include <limits>
#include <new>
#include <string>
#include <utility>
#include <vector>

namespace {

constexpr CLSID kProviderClsid = {0x64a6a7bf, 0xba56, 0x4463,
    {0x96, 0x92, 0x83, 0xa4, 0xea, 0x3d, 0xbc, 0x6c}};
constexpr DWORD kHostTimeoutMs = 20000;
constexpr size_t kMaxHostOutput = 256;
std::atomic<long> g_objects{0};
std::atomic<long> g_locks{0};

enum Field : DWORD {
    kTitle,
    kInstructions,
    kRiAuthPassword,
    kOtp,
    kWindowsPassword,
    kSubmit,
    kFieldCount
};

struct FieldDefinition {
    CREDENTIAL_PROVIDER_FIELD_TYPE type;
    const wchar_t* label;
};

constexpr FieldDefinition kFields[kFieldCount] = {
    {CPFT_LARGE_TEXT, L"riAuth"},
    {CPFT_SMALL_TEXT, L"Online approval for the enrolled local Windows account"},
    {CPFT_PASSWORD_TEXT, L"riAuth password"},
    {CPFT_PASSWORD_TEXT, L"One-time code (if required)"},
    {CPFT_PASSWORD_TEXT, L"Local Windows password"},
    {CPFT_SUBMIT_BUTTON, L"Sign in"},
};

HRESULT Duplicate(const wchar_t* value, wchar_t** out) {
    if (!out || !value) return E_INVALIDARG;
    *out = nullptr;
    const size_t length = wcslen(value);
    if (length > (std::numeric_limits<size_t>::max() / sizeof(wchar_t)) - 1)
        return E_OUTOFMEMORY;
    auto* copy = static_cast<wchar_t*>(CoTaskMemAlloc((length + 1) * sizeof(wchar_t)));
    if (!copy) return E_OUTOFMEMORY;
    memcpy(copy, value, (length + 1) * sizeof(wchar_t));
    *out = copy;
    return S_OK;
}

void Wipe(std::wstring& value) {
    if (!value.empty()) SecureZeroMemory(value.data(), value.size() * sizeof(wchar_t));
    value.clear();
}

void Wipe(std::string& value) {
    if (!value.empty()) SecureZeroMemory(value.data(), value.size());
    value.clear();
}

class Handle {
public:
    HANDLE value = nullptr;
    Handle() = default;
    explicit Handle(HANDLE h) : value(h) {}
    Handle(const Handle&) = delete;
    Handle& operator=(const Handle&) = delete;
    ~Handle() { Reset(); }
    void Reset(HANDLE next = nullptr) {
        if (value && value != INVALID_HANDLE_VALUE) CloseHandle(value);
        value = next;
    }
    HANDLE Release() { auto* h = value; value = nullptr; return h; }
    explicit operator bool() const { return value && value != INVALID_HANDLE_VALUE; }
};

bool HostPath(std::wstring& path) {
    wchar_t* programFiles = nullptr;
    if (FAILED(SHGetKnownFolderPath(FOLDERID_ProgramFiles, KF_FLAG_DEFAULT,
                                    nullptr, &programFiles))) return false;
    path.assign(programFiles);
    CoTaskMemFree(programFiles);
    path += L"\\RiAuth\\DeviceHost\\RiAuth.DeviceHost.exe";
    const DWORD attributes = GetFileAttributesW(path.c_str());
    return attributes != INVALID_FILE_ATTRIBUTES &&
           !(attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT));
}

bool ToUtf8(const std::wstring& input, std::string& output) {
    if (input.size() > static_cast<size_t>(std::numeric_limits<int>::max())) return false;
    const int length = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS,
                                          input.data(), static_cast<int>(input.size()),
                                          nullptr, 0, nullptr, nullptr);
    if (length <= 0 && !input.empty()) return false;
    output.resize(static_cast<size_t>(length));
    if (length && WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS,
                                     input.data(), static_cast<int>(input.size()),
                                     output.data(), length, nullptr, nullptr) != length) {
        Wipe(output);
        return false;
    }
    return true;
}

bool AppendJsonString(const std::wstring& input, std::string& output) {
    std::string utf8;
    if (!ToUtf8(input, utf8)) return false;
    output.push_back('"');
    constexpr char kHex[] = "0123456789abcdef";
    for (const unsigned char ch : utf8) {
        if (ch == '"' || ch == '\\') {
            output.push_back('\\');
            output.push_back(static_cast<char>(ch));
        } else if (ch < 0x20) {
            output.append("\\u00");
            output.push_back(kHex[ch >> 4]);
            output.push_back(kHex[ch & 15]);
        } else {
            output.push_back(static_cast<char>(ch));
        }
    }
    output.push_back('"');
    Wipe(utf8);
    return true;
}

bool LoginInput(const std::wstring& password, const std::wstring& otp,
                std::string& input) {
    input.assign("{\"password\":");
    if (!AppendJsonString(password, input)) return false;
    input.append(",\"otp\":");
    if (otp.empty()) input.append("null");
    else if (!AppendJsonString(otp, input)) return false;
    input.append("}\n");
    return input.size() <= 8192;
}

struct WriterContext {
    HANDLE pipe;
    const std::string* input;
    bool success = false;
};

DWORD WINAPI WriteInput(void* raw) {
    auto* context = static_cast<WriterContext*>(raw);
    size_t offset = 0;
    while (offset < context->input->size()) {
        DWORD written = 0;
        const DWORD remaining = static_cast<DWORD>(context->input->size() - offset);
        if (!WriteFile(context->pipe, context->input->data() + offset,
                       remaining, &written, nullptr) || written == 0) break;
        offset += written;
    }
    context->success = offset == context->input->size();
    CloseHandle(context->pipe); // EOF tells the host there is no second proof.
    return 0;
}

// The host emits a single small, exact approval record. Nonzero exit, timeout,
// malformed stdout, or any extra stdout/stderr are always a denial.
bool InvokeHost(const wchar_t* command, const std::string* input,
                const char* marker, std::wstring& sidText) {
    std::wstring executable;
    if (!HostPath(executable)) return false;

    SECURITY_ATTRIBUTES security{sizeof(security), nullptr, TRUE};
    Handle childInput, parentInput, parentOutput, childOutput;
    if (!CreatePipe(&childInput.value, &parentInput.value, &security, 16384) ||
        !CreatePipe(&parentOutput.value, &childOutput.value, &security, 512) ||
        !SetHandleInformation(parentInput.value, HANDLE_FLAG_INHERIT, 0) ||
        !SetHandleInformation(parentOutput.value, HANDLE_FLAG_INHERIT, 0)) return false;

    SIZE_T listSize = 0;
    InitializeProcThreadAttributeList(nullptr, 1, 0, &listSize);
    if (!listSize) return false;
    std::vector<unsigned char> listStorage(listSize);
    auto* attributeList = reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(listStorage.data());
    if (!InitializeProcThreadAttributeList(attributeList, 1, 0, &listSize)) return false;
    HANDLE inherited[] = {childInput.value, childOutput.value};
    const bool listReady = UpdateProcThreadAttribute(attributeList, 0,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, inherited, sizeof(inherited), nullptr, nullptr) != 0;
    if (!listReady) {
        DeleteProcThreadAttributeList(attributeList);
        return false;
    }

    STARTUPINFOEXW startup{};
    startup.StartupInfo.cb = sizeof(startup);
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = childInput.value;
    startup.StartupInfo.hStdOutput = childOutput.value;
    startup.StartupInfo.hStdError = childOutput.value;
    startup.lpAttributeList = attributeList;
    std::wstring commandLine = L"\"" + executable + L"\" "+ command;
    PROCESS_INFORMATION process{};
    const BOOL started = CreateProcessW(executable.c_str(), commandLine.data(),
        nullptr, nullptr, TRUE,
        CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
        nullptr, nullptr, &startup.StartupInfo, &process);
    DeleteProcThreadAttributeList(attributeList);
    if (!started) return false;
    Handle processHandle(process.hProcess), threadHandle(process.hThread);
    childInput.Reset();
    childOutput.Reset();

    WriterContext writer{parentInput.value, input};
    Handle writerThread;
    if (input) {
        parentInput.Release(); // The writer closes the pipe after the proof.
        writerThread.Reset(CreateThread(nullptr, 0, WriteInput, &writer, 0, nullptr));
        if (!writerThread) {
            CloseHandle(writer.pipe);
            TerminateProcess(processHandle.value, 1);
            WaitForSingleObject(processHandle.value, INFINITE);
            return false;
        }
    } else {
        parentInput.Reset();
    }

    const DWORD wait = WaitForSingleObject(processHandle.value, kHostTimeoutMs);
    if (wait != WAIT_OBJECT_0) {
        TerminateProcess(processHandle.value, 1);
        WaitForSingleObject(processHandle.value, INFINITE);
    }
    if (writerThread) WaitForSingleObject(writerThread.value, INFINITE);
    if (wait != WAIT_OBJECT_0 || (input && !writer.success)) return false;

    DWORD exitCode = 0;
    if (!GetExitCodeProcess(processHandle.value, &exitCode) || exitCode != 0) return false;
    DWORD available = 0;
    if (!PeekNamedPipe(parentOutput.value, nullptr, 0, nullptr, &available, nullptr) ||
        available == 0 || available > kMaxHostOutput) return false;
    std::string output(available, '\0');
    DWORD read = 0;
    if (!ReadFile(parentOutput.value, output.data(), available, &read, nullptr) ||
        read != available) return false;

    const std::string prefix = std::string(marker) + "\n";
    if (output.compare(0, prefix.size(), prefix) != 0 ||
        output.size() <= prefix.size() + 2 || output.back() != '\n') return false;
    const std::string sid = output.substr(prefix.size(), output.size() - prefix.size() - 1);
    if (sid.size() > 128 || sid.compare(0, 6, "S-1-5-") != 0) return false;
    for (const char ch : sid) {
        if ((ch < '0' || ch > '9') && ch != 'S' && ch != '-') return false;
    }
    sidText.assign(sid.begin(), sid.end());
    return true;
}

bool LocalAccountForSid(const std::wstring& sidText,
                        std::wstring& domain, std::wstring& username) {
    PSID sid = nullptr;
    if (!ConvertStringSidToSidW(sidText.c_str(), &sid)) return false;
    DWORD nameLength = 0, domainLength = 0;
    SID_NAME_USE use{};
    LookupAccountSidW(nullptr, sid, nullptr, &nameLength, nullptr, &domainLength, &use);
    if (GetLastError() != ERROR_INSUFFICIENT_BUFFER || !nameLength || !domainLength ||
        nameLength > 256 || domainLength > 256) {
        LocalFree(sid);
        return false;
    }
    std::vector<wchar_t> name(nameLength), authority(domainLength);
    const bool resolved = LookupAccountSidW(nullptr, sid, name.data(), &nameLength,
                                             authority.data(), &domainLength, &use) != 0;
    LocalFree(sid);
    if (!resolved || use != SidTypeUser) return false;

    wchar_t computer[MAX_COMPUTERNAME_LENGTH + 1]{};
    DWORD computerLength = MAX_COMPUTERNAME_LENGTH + 1;
    if (!GetComputerNameW(computer, &computerLength) ||
        _wcsicmp(authority.data(), computer) != 0) return false;

    LPBYTE userInfo = nullptr;
    const NET_API_STATUS status = NetUserGetInfo(nullptr, name.data(), 1, &userInfo);
    if (status != NERR_Success || !userInfo) {
        if (userInfo) NetApiBufferFree(userInfo);
        return false;
    }
    const auto* user = reinterpret_cast<USER_INFO_1*>(userInfo);
    const bool enabled = (user->usri1_flags & (UF_ACCOUNTDISABLE | UF_LOCKOUT)) == 0;
    NetApiBufferFree(userInfo);
    if (!enabled) return false;
    // The account name is not the binding. Re-resolve it against the local SAM
    // and compare its SID so a rename/recreate or foreign-domain collision fails.
    const std::wstring qualified = std::wstring(computer) + L"\\" + name.data();
    DWORD localSidLength = 0, localDomainLength = 0;
    SID_NAME_USE localUse{};
    LookupAccountNameW(nullptr, qualified.c_str(), nullptr, &localSidLength,
                       nullptr, &localDomainLength, &localUse);
    if (GetLastError() != ERROR_INSUFFICIENT_BUFFER || !localSidLength ||
        localSidLength > SECURITY_MAX_SID_SIZE || !localDomainLength ||
        localDomainLength > 256) return false;
    std::vector<BYTE> localSid(localSidLength);
    std::vector<wchar_t> localDomain(localDomainLength);
    if (!LookupAccountNameW(nullptr, qualified.c_str(), localSid.data(), &localSidLength,
                            localDomain.data(), &localDomainLength, &localUse) ||
        localUse != SidTypeUser || _wcsicmp(localDomain.data(), computer) != 0)
        return false;
    PSID pinnedSid = nullptr;
    if (!ConvertStringSidToSidW(sidText.c_str(), &pinnedSid)) return false;
    const bool sameSid = EqualSid(pinnedSid, localSid.data()) != 0;
    LocalFree(pinnedSid);
    if (!sameSid) return false;
    domain.assign(authority.data());
    username.assign(name.data());
    return !username.empty();
}

bool NegotiatePackage(ULONG& id) {
    LSA_HANDLE lsa = nullptr;
    if (LsaConnectUntrusted(&lsa) < 0) return false;
    char name[] = NEGOSSP_NAME_A;
    LSA_STRING package{static_cast<USHORT>(strlen(name)),
                       static_cast<USHORT>(sizeof(name)), name};
    const NTSTATUS status = LsaLookupAuthenticationPackage(lsa, &package, &id);
    LsaDeregisterLogonProcess(lsa);
    return status >= 0;
}

bool ProtectPassword(const std::wstring& input, std::wstring& protectedPassword) {
    if (input.empty() || input.size() > 1024) return false;
    std::wstring mutableInput = input;
    DWORD length = 0;
    CredProtectW(FALSE, mutableInput.data(), static_cast<DWORD>(input.size() + 1),
                 nullptr, &length, nullptr);
    if (GetLastError() != ERROR_INSUFFICIENT_BUFFER || length == 0 || length > 4096) {
        Wipe(mutableInput);
        return false;
    }
    protectedPassword.resize(length);
    const bool success = CredProtectW(FALSE, mutableInput.data(),
        static_cast<DWORD>(input.size() + 1), protectedPassword.data(), &length, nullptr) != 0;
    Wipe(mutableInput);
    if (!success || length == 0) {
        Wipe(protectedPassword);
        return false;
    }
    protectedPassword.resize(length - 1); // CredProtect's length includes NUL.
    return true;
}

bool PackCredential(CREDENTIAL_PROVIDER_USAGE_SCENARIO scenario,
                    const std::wstring& domain, const std::wstring& username,
                    const std::wstring& password,
                    CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION& output) {
    if (scenario != CPUS_LOGON && scenario != CPUS_UNLOCK_WORKSTATION) return false;
    const size_t domainBytes = domain.size() * sizeof(wchar_t);
    const size_t userBytes = username.size() * sizeof(wchar_t);
    const size_t passwordBytes = password.size() * sizeof(wchar_t);
    if (domainBytes > USHRT_MAX || userBytes > USHRT_MAX || passwordBytes > USHRT_MAX)
        return false;
    const size_t length = sizeof(KERB_INTERACTIVE_UNLOCK_LOGON) +
                          domainBytes + userBytes + passwordBytes;
    if (length > MAXDWORD) return false;
    auto* bytes = static_cast<BYTE*>(CoTaskMemAlloc(length));
    if (!bytes) return false;
    SecureZeroMemory(bytes, length);
    auto* packed = reinterpret_cast<KERB_INTERACTIVE_UNLOCK_LOGON*>(bytes);
    packed->Logon.MessageType = scenario == CPUS_LOGON
        ? KerbInteractiveLogon : KerbWorkstationUnlockLogon;
    size_t offset = sizeof(*packed);
    const auto add = [&](const std::wstring& value, UNICODE_STRING& field) {
        const size_t count = value.size() * sizeof(wchar_t);
        field.Length = static_cast<USHORT>(count);
        field.MaximumLength = field.Length;
        field.Buffer = reinterpret_cast<PWSTR>(offset);
        if (count) memcpy(bytes + offset, value.data(), count);
        offset += count;
    };
    add(domain, packed->Logon.LogonDomainName);
    add(username, packed->Logon.UserName);
    add(password, packed->Logon.Password);

    ULONG package = 0;
    if (!NegotiatePackage(package)) {
        SecureZeroMemory(bytes, length);
        CoTaskMemFree(bytes);
        return false;
    }
    output.ulAuthenticationPackage = package;
    output.clsidCredentialProvider = kProviderClsid;
    output.cbSerialization = static_cast<ULONG>(length);
    output.rgbSerialization = bytes;
    return true;
}

class Credential final : public ICredentialProviderCredential2 {
public:
    Credential(CREDENTIAL_PROVIDER_USAGE_SCENARIO scenario, std::wstring tileSid)
        : scenario_(scenario), tileSid_(std::move(tileSid)) {
        ++g_objects;
    }
    ~Credential() {
        ClearSecrets();
        --g_objects;
    }

    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** out) override {
        if (!out) return E_POINTER;
        *out = nullptr;
        if (IsEqualIID(iid, IID_IUnknown) ||
            IsEqualIID(iid, IID_ICredentialProviderCredential) ||
            IsEqualIID(iid, IID_ICredentialProviderCredential2)) {
            *out = static_cast<ICredentialProviderCredential2*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return ++references_; }
    ULONG STDMETHODCALLTYPE Release() override {
        const ULONG remaining = --references_;
        if (!remaining) delete this;
        return remaining;
    }

    HRESULT STDMETHODCALLTYPE Advise(ICredentialProviderCredentialEvents*) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE UnAdvise() override { return S_OK; }
    HRESULT STDMETHODCALLTYPE SetSelected(BOOL* autoLogon) override {
        if (!autoLogon) return E_POINTER;
        *autoLogon = FALSE;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE SetDeselected() override { ClearSecrets(); return S_OK; }
    HRESULT STDMETHODCALLTYPE GetFieldState(DWORD field,
        CREDENTIAL_PROVIDER_FIELD_STATE* state,
        CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE* interactive) override {
        if (!state || !interactive) return E_POINTER;
        if (field >= kFieldCount) return E_INVALIDARG;
        *state = field == kTitle ? CPFS_DISPLAY_IN_BOTH : CPFS_DISPLAY_IN_SELECTED_TILE;
        *interactive = field == kRiAuthPassword ? CPFIS_FOCUSED : CPFIS_NONE;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetStringValue(DWORD field, wchar_t** value) override {
        if (!value) return E_POINTER;
        if (field >= kFieldCount) return E_INVALIDARG;
        // LogonUI owns the input fields; do not repopulate a secret after it is cleared.
        return Duplicate(field == kTitle || field == kInstructions
                             ? kFields[field].label : L"", value);
    }
    HRESULT STDMETHODCALLTYPE GetBitmapValue(DWORD, HBITMAP*) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE GetCheckboxValue(DWORD, BOOL*, wchar_t**) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE GetSubmitButtonValue(DWORD field, DWORD* adjacent) override {
        if (!adjacent) return E_POINTER;
        if (field != kSubmit) return E_INVALIDARG;
        *adjacent = kWindowsPassword;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetComboBoxValueCount(DWORD, DWORD*, DWORD*) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE GetComboBoxValueAt(DWORD, DWORD, wchar_t**) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE SetStringValue(DWORD field, const wchar_t* value) override {
        std::wstring* target = nullptr;
        size_t limit = 0;
        switch (field) {
            case kRiAuthPassword: target = &riAuthPassword_; limit = 1024; break;
            case kOtp: target = &otp_; limit = 128; break;
            case kWindowsPassword: target = &windowsPassword_; limit = 1024; break;
            default: return E_INVALIDARG;
        }
        if (!value) {
            Wipe(*target);
            return E_POINTER;
        }
        size_t length = 0;
        while (length <= limit && value[length]) ++length;
        if (length > limit) {
            Wipe(*target);
            return E_INVALIDARG;
        }
        Wipe(*target);
        target->assign(value, length);
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE SetCheckboxValue(DWORD, BOOL) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE SetComboBoxSelectedValue(DWORD, DWORD) override { return E_INVALIDARG; }
    HRESULT STDMETHODCALLTYPE CommandLinkClicked(DWORD) override { return E_INVALIDARG; }

    HRESULT STDMETHODCALLTYPE GetUserSid(wchar_t** sid) override {
        if (!sid) return E_POINTER;
        *sid = nullptr;
        std::wstring pinned, domain, user;
        if (tileSid_.empty() ||
            !InvokeHost(L"cp-account", nullptr, "RIAUTH-CP-ACCOUNT-V1", pinned) ||
            _wcsicmp(tileSid_.c_str(), pinned.c_str()) != 0 ||
            !LocalAccountForSid(pinned, domain, user)) return E_FAIL;
        return Duplicate(tileSid_.c_str(), sid);
    }

    HRESULT STDMETHODCALLTYPE GetSerialization(
        CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE* response,
        CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION* serialization,
        wchar_t** statusText, CREDENTIAL_PROVIDER_STATUS_ICON* statusIcon) override {
        if (!response || !serialization || !statusText || !statusIcon) return E_POINTER;
        *response = CPGSR_NO_CREDENTIAL_NOT_FINISHED;
        *statusText = nullptr;
        *statusIcon = CPSI_NONE;
        SecureZeroMemory(serialization, sizeof(*serialization));

        bool approved = false;
        if ((scenario_ == CPUS_LOGON || scenario_ == CPUS_UNLOCK_WORKSTATION) &&
            !tileSid_.empty() &&
            !riAuthPassword_.empty() && !windowsPassword_.empty()) {
            std::string proof;
            if (LoginInput(riAuthPassword_, otp_, proof)) {
                std::wstring sid, domain, username;
                approved = InvokeHost(L"cp-login --proof-stdin", &proof,
                    "RIAUTH-CP-APPROVED-V1", sid) &&
                    _wcsicmp(tileSid_.c_str(), sid.c_str()) == 0 &&
                    LocalAccountForSid(sid, domain, username);
                if (approved) {
                    std::wstring protectedPassword;
                    approved = ProtectPassword(windowsPassword_, protectedPassword) &&
                        PackCredential(scenario_, domain, username,
                                       protectedPassword, *serialization);
                    Wipe(protectedPassword);
                }
            }
            Wipe(proof);
        }
        ClearSecrets();
        if (approved) {
            *response = CPGSR_RETURN_CREDENTIAL_FINISHED;
            return S_OK;
        }
        Duplicate(L"riAuth approval unavailable or denied. Check the connection and credentials.",
                  statusText);
        *statusIcon = CPSI_ERROR;
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE ReportResult(NTSTATUS status, NTSTATUS,
        wchar_t** text, CREDENTIAL_PROVIDER_STATUS_ICON* icon) override {
        ClearSecrets();
        if (!text || !icon) return E_POINTER;
        *text = nullptr;
        *icon = CPSI_NONE;
        if (status < 0) {
            Duplicate(L"Windows rejected the local account credentials.", text);
            *icon = CPSI_ERROR;
        }
        return S_OK;
    }

private:
    void ClearSecrets() {
        Wipe(riAuthPassword_);
        Wipe(otp_);
        Wipe(windowsPassword_);
    }
    std::atomic<ULONG> references_{1};
    CREDENTIAL_PROVIDER_USAGE_SCENARIO scenario_;
    std::wstring riAuthPassword_, otp_, windowsPassword_, tileSid_;
};

class Provider final : public ICredentialProvider {
public:
    Provider() { ++g_objects; }
    ~Provider() { --g_objects; }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** out) override {
        if (!out) return E_POINTER;
        *out = nullptr;
        if (IsEqualIID(iid, IID_IUnknown) || IsEqualIID(iid, IID_ICredentialProvider)) {
            *out = static_cast<ICredentialProvider*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return ++references_; }
    ULONG STDMETHODCALLTYPE Release() override {
        const ULONG remaining = --references_;
        if (!remaining) delete this;
        return remaining;
    }
    HRESULT STDMETHODCALLTYPE SetUsageScenario(CREDENTIAL_PROVIDER_USAGE_SCENARIO scenario,
                                               DWORD) override {
        if (scenario != CPUS_LOGON && scenario != CPUS_UNLOCK_WORKSTATION)
            return E_NOTIMPL;
        scenario_ = scenario;
        tileSid_.clear();
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE SetSerialization(
        const CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION*) override {
        // Never accept caller-provided credentials as riAuth approval.
        return E_NOTIMPL;
    }
    HRESULT STDMETHODCALLTYPE Advise(ICredentialProviderEvents*, UINT_PTR) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE UnAdvise() override { return S_OK; }
    HRESULT STDMETHODCALLTYPE GetFieldDescriptorCount(DWORD* count) override {
        if (!count) return E_POINTER;
        *count = kFieldCount;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetFieldDescriptorAt(DWORD index,
        CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR** descriptor) override {
        if (!descriptor) return E_POINTER;
        *descriptor = nullptr;
        if (index >= kFieldCount) return E_INVALIDARG;
        auto* field = static_cast<CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR*>(
            CoTaskMemAlloc(sizeof(CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR)));
        if (!field) return E_OUTOFMEMORY;
        SecureZeroMemory(field, sizeof(*field));
        field->dwFieldID = index;
        field->cpft = kFields[index].type;
        const HRESULT result = Duplicate(kFields[index].label, &field->pszLabel);
        if (FAILED(result)) {
            CoTaskMemFree(field);
            return result;
        }
        *descriptor = field;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetCredentialCount(DWORD* count, DWORD* defaultIndex,
                                                  BOOL* autoLogon) override {
        if (!count || !defaultIndex || !autoLogon) return E_POINTER;
        tileSid_.clear();
        std::wstring candidate, domain, username;
        if ((scenario_ == CPUS_LOGON || scenario_ == CPUS_UNLOCK_WORKSTATION) &&
            InvokeHost(L"cp-account", nullptr, "RIAUTH-CP-ACCOUNT-V1", candidate) &&
            LocalAccountForSid(candidate, domain, username)) tileSid_ = std::move(candidate);
        *count = tileSid_.empty() ? 0 : 1;
        *defaultIndex = CREDENTIAL_PROVIDER_NO_DEFAULT;
        *autoLogon = FALSE;
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetCredentialAt(DWORD index,
        ICredentialProviderCredential** credential) override {
        if (!credential) return E_POINTER;
        *credential = nullptr;
        if (index != 0 || tileSid_.empty() ||
            (scenario_ != CPUS_LOGON && scenario_ != CPUS_UNLOCK_WORKSTATION))
            return E_INVALIDARG;
        auto* next = new (std::nothrow) Credential(scenario_, tileSid_);
        if (!next) return E_OUTOFMEMORY;
        *credential = static_cast<ICredentialProviderCredential*>(next);
        return S_OK;
    }
private:
    std::atomic<ULONG> references_{1};
    CREDENTIAL_PROVIDER_USAGE_SCENARIO scenario_ = CPUS_INVALID;
    std::wstring tileSid_;
};

class Factory final : public IClassFactory {
public:
    Factory() { ++g_objects; }
    ~Factory() { --g_objects; }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** out) override {
        if (!out) return E_POINTER;
        *out = nullptr;
        if (IsEqualIID(iid, IID_IUnknown) || IsEqualIID(iid, IID_IClassFactory)) {
            *out = static_cast<IClassFactory*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return ++references_; }
    ULONG STDMETHODCALLTYPE Release() override {
        const ULONG remaining = --references_;
        if (!remaining) delete this;
        return remaining;
    }
    HRESULT STDMETHODCALLTYPE CreateInstance(IUnknown* outer, REFIID iid, void** out) override {
        if (!out) return E_POINTER;
        *out = nullptr;
        if (outer) return CLASS_E_NOAGGREGATION;
        auto* provider = new (std::nothrow) Provider();
        if (!provider) return E_OUTOFMEMORY;
        const HRESULT result = provider->QueryInterface(iid, out);
        provider->Release();
        return result;
    }
    HRESULT STDMETHODCALLTYPE LockServer(BOOL lock) override {
        if (lock) ++g_locks;
        else --g_locks;
        return S_OK;
    }
private:
    std::atomic<ULONG> references_{1};
};

} // namespace

extern "C" BOOL WINAPI DllMain(HINSTANCE module, DWORD reason, void*) {
    if (reason == DLL_PROCESS_ATTACH) DisableThreadLibraryCalls(module);
    return TRUE;
}

extern "C" HRESULT STDMETHODCALLTYPE
DllGetClassObject(REFCLSID clsid, REFIID iid, void** out) {
    if (!out) return E_POINTER;
    *out = nullptr;
    if (!IsEqualCLSID(clsid, kProviderClsid)) return CLASS_E_CLASSNOTAVAILABLE;
    auto* factory = new (std::nothrow) Factory();
    if (!factory) return E_OUTOFMEMORY;
    const HRESULT result = factory->QueryInterface(iid, out);
    factory->Release();
    return result;
}

extern "C" HRESULT STDMETHODCALLTYPE DllCanUnloadNow() {
    return g_objects == 0 && g_locks == 0 ? S_OK : S_FALSE;
}
