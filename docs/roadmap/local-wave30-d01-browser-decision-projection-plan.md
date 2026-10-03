# D01 fixed public decision projection — source plan only

Date: 2026-10-03, Europe/Vaduz. Reservation `wave30_D01_public_decision_projection_source_plan`; project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, supporting existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` only. Primary WT42 is unchanged. Entry remains `674e04874bbcfa1536507e28a5fec629a86ceb72`; root supplied publication `6e949b97573d7a4cc42cf5ac1ec43c5fc3963599`. No alignment/import/merge. The sole tracked write is this new report; the complete candidate is an archive, not a materialized executable cell or source helper.

The source-backed omission is narrow: page stores fresh ref strings but no fixed public label/ref association; successful output publishes only page flags. The exact predicate whitelist also omits required-consent `Allow`, the configured-client consent heading, and the source's exact OTP label. A caller following the source-backed password UI cannot declare those expected public observations using the current whitelist. This is an observational/decision seam, not evidence of a historical sender, lost password, port/provider issue, exception cause or completed journey. All historical failed fixtures and UNKNOWN cause/sender/true first-cleanup timing remain unchanged.

## Immutable bodies read and exact witness spans

I read the complete 485-line candidate at `9f3a4a372b53fc6d73d5df45ad1a8d217ad60879:docs/roadmap/local-wave30-d01-continuation-correction-plan.md` (fence document lines 227–711), not just its hash. Candidate lines 311–342 implement page flags and fresh refs; 386–433 retain the exact continuation, password/input wrappers and single-use ref; 434–445 hold the old predicate; 467–485 hold final output. The five collectors, first-failure/controller observation, phase/drain, owned cleanup and input bodies were fully read and stay exact.

I read the complete 225-line controller command at `2e29c30d01ccd41a2aac3914e3ac20afa38d324f:docs/roadmap/local-wave30-d01-user-browser-review.md`, fence document lines 7698–7922. Its canonical bytes exclude the one delimiter newline after final `PY`: `17326` bytes, SHA-256 `5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0`. Controller line 157 calls client create with exact client ID `local-demo`, `--name 'Local demo'`, confidential mode, the fixed callback, `openid,profile` and private secret file. Lines 145–176 fix the disposable operator/server/helper setup and 180-second preparation/600-second helper bounds. No controller is imported or executed, and no private generated value is read.

The full pinned `c01c39ab4e092423d5522bedc50fff87656d8c0a` signin HTML and JS were read, along with full shared `auth.js` transport. HTML lines 37–40 have Username, Password, exact `Authenticator or recovery code (if enabled)` and Sign in; line 59 has Allow/Deny. JS line 10 fixes `Sign in to continue to {App}`; line 40 derives app from `state.application.name`; line 186 fixes the ordinary OTP label, line 216 fixes required consent `${app()} wants to use your riAuth account` versus `Continue to ${app()}?`, and line 229 fixes Allow versus Continue. JS lines 309–322 handle password form submission; lines 298–304/350–351 send guarded consent decisions. No terminal approval, direct decision POST, private fields, or state export is introduced.

Selected pinned `src/api/interaction.rs` lines 25–44, 84–110, 117–166 and 247–282 were read: interaction/state/password/decision routing, binding cookie, portal HTML and browser-write/credential guards. Selected `src/assembly/browser_runtime.rs` lines 385–398 and 1130–1180 were read: authorize_state resolves the bound browser interaction; status_for resolves client name and inserts it into `application.name`. The state includes account/session_ref/host/continue data, which this proposal never reads or exports. The parsed JS transport retains same-origin cookies, Origin/header protections, retry boundaries and no token/cookie reads. Source routing is context only; no private resume URL/ID/query is projected.

The full 757-line fixed descriptor helper was read, not imported: `f4ef05d8428b235511e81277ba6b4b72d5ec08ba:scripts/d01-confidential-browser-demo.py`, blob `a01b1f3f81f0978eb0a02ed339c7248cae485350`, `35749` bytes / SHA-256 `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`. Checked-out bytes equal that object. Its fixed HTML in Handler.reply, lines 575–582, prints Local demo, Sign in, Protected application access and the fixed status text, while never printing subject. Handler.get's protected-before/after checks and main's terminal/evidence predicates remain protected. All parsing, HTTP/header/authority/Authorization-four-counter, receipt/secret, nonce/state/S256/signature/provider/native/cleanup routines remain unchanged.

| c01 source identity | Git blob | Bytes / LF lines | SHA-256 |
| --- | --- | --- | --- |
| `src/portal/signin.html` | `e3e120898822bdfe331e4dc2d3fdeb59be048b08` | `6470 / 80` | `af5c163944bb397d6649c932fa91e749381772d307c685a95f53cd3f6dee5f44` |
| `src/portal/signin.js` | `af602f2449f83a7489f5c8f6aa47d737dfc523c8` | `23197 / 373` | `3d5ee4cfc1b3a88b3d6fa46d064df36b7f1a227d03a1a557830eac4528ccff22` |
| `src/portal/auth.js` | `e07260923ebb34995ef4d31a1b4605980626df7c` | `9235 / 175` | `c1a1aa440ca0144183019959b94bf950095309cbc414ec7bcffefbf48cf63ae3` |
| `src/api/interaction.rs` | `d4714c7590b251e8d94589a6b0b4f6f1e7bc32bd` | `10192 / 311` | `d4ed262fac0bbd167c3dd999f260ce7b26e9c61bcb5072bb0b380028cc2b38b1` |
| `src/assembly/browser_runtime.rs` | `f5ba027be118057b8ff491d168205f283caec4ac` | `72437 / 1740` | `d0548f68ef985fc2b59f72015bca4fe957e89895cc8c870ff48f07b6a5f3c16f` |

These are Git source identities; no c01 snapshot, artifact, browser, label/ref or current UI observation is credited by these reads.

## One surgical candidate seam and root's next decision

The proposal has three textual anchors: add one projection assignment immediately after the unchanged fresh-ref extraction; replace only the old publicPredicate span with one pure projector plus the narrowed exact-pair predicate; add only a `public_decision` property to successful output. The projector's twelve exact role/name pairs are printed in its full source below. It exports `{observed:[{role,name}],refs:[{role,name,action,ref}]}` and no other snapshot field. Observed pairs are deduplicated against the fixed declaration; copied action role/name strings also come from that canonical declaration, and each provider ref is captured once before namespace validation. Action refs preserve provider order without ranking, geometry, index choice or automatic selection. Root must reject an ambiguous action association rather than pick the first.

Required consent has the fixed Local demo heading and Allow; optional consent has Continue to Local demo? and Continue. Both possibilities are admitted as observations, not promises that this fixture reaches either state. Exact Username/Password/OTP and Sign in labels come from the pinned UI. OTP is observed by role/name only: no value, empty/nonempty test or actionable ref is exported, and no OTP fill is added. Fixed RP heading/status pairs remain observable.

For every action association the role/name must match the fixed table, the ref must be a provider `p<snapshot>:<index>` string matching the already accepted namespace regex, and the SAME result's typed ref must advertise `click` for a fixed button or `type` for Username/Password. There is no label-to-ref join across two snapshots, content-array index association, inferred field value or fall back to geometry. Unmatched ref roles/names, a missing required advertised action, or a malformed ref discard that association; extra provider action strings are never copied. Tool error, an incomplete snapshot, non-ok status, or the fixed RP error label yields an empty projection. Future native provider output may omit a pair/capability; absence is not silently repaired. This phase does not independently attest a nonblank provider shape or evaluate the archived fixture stubs.

Current callable tool descriptions were read as static metadata only: get_browser_state semantic_v2 returns typed action/content refs; browser click/type describe the p namespace and invalidation on navigation/newer snapshots. Historical report contract prose was read for context; no old snapshot is reused. No Driver method was called, and no provider/session state was inspected. The existing snapshotArgs bind exact owned session/target/tab. The original fresh_refs extraction, inclusion check, clear-before-input and provider invalidation are unchanged. The new projection neither authorizes an action nor replaces that ref guard. Root maps a returned type association to the already existing username/password decision; there is no new dispatcher kind or private transfer adapter. A caller must choose from the latest successful projection of the same owned snapshot, pin the exact expected role/name before dispatch, and refresh after each single-use input. Old refs after navigation/newer snapshot or a stopped fixture remain unusable under existing guards.

The page and predicate use the same exact-pair projection. The old role/label Cartesian product is narrowed to source-backed pairs, including replacement of the obsolete OTP wording; arbitrary labels, unrelated clients, private account headings, errors, URL/query/headers/cookies/state and HTML are excluded. Projection is observational, not proof that synthetic dom_event activation or type dispatch succeeded. The unchanged continuation still verifies its next expected snapshot, retains first failure and cleanup, and earns no journey credit merely from dispatch. Root selects the next action; this source plan does not do it.

## Exact prospective source identities and diff

Immutable 9f candidate: 32502 bytes / 485 lines / SHA-256 `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8`.

Proposed complete candidate: 33701 bytes / 508 lines / SHA-256 `505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f`.

Forward diff: 2405 bytes / 46 lines / SHA-256 `b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2`.

Independent inverse: 2405 bytes / 46 lines / SHA-256 `3a9053840f82174e4a761cd319b04e82bf2c62c2cc31adccb4a320804e968ae2`.

```diff
--- a/archived-d01-continuation-cell.js
+++ b/archived-d01-continuation-cell.js
@@ -328,0 +329 @@
+  own.public_decision=projectPublicDecision(result);
@@ -433,0 +435,27 @@
+function projectPublicDecision(result) {
+  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
+  const labels=[
+    ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
+    ["heading","Local demo wants to use your riAuth account"],
+    ["heading","Continue to Local demo?"],["heading","Protected application access"],
+    ["button","Sign in"],["button","Allow"],["button","Continue"],
+    ["textbox","Username"],["textbox","Password"],
+    ["textbox","Authenticator or recovery code (if enabled)"],
+    ["statictext","Signed in. Protected application access is available."]
+  ];
+  const valid=result?.isError!==true&&s?.status==="ok"&&s?.snapshot?.complete===true&&
+    !nodes.some(n=>n.name==="Local demo could not complete this request.");
+  return {
+    observed:valid?labels.filter(([role,name])=>
+      nodes.some(n=>n.role===role&&n.name===name)).map(([role,name])=>({role,name})):[],
+    refs:valid&&Array.isArray(s?.refs)?s.refs.flatMap(n=>{
+      const pair=labels.find(([role,name])=>n?.role===role&&n.name===name),ref=n?.ref;
+      if(!pair||typeof ref!=="string"||!/^p[0-9]+:[0-9]+$/.test(ref))return [];
+      const [role,name]=pair;
+      const action=role==="button"?"click":
+        role==="textbox"&&["Username","Password"].includes(name)?"type":null;
+      if(action===null||!Array.isArray(n.actions)||!n.actions.includes(action))return [];
+      return [{role,name,action,ref}];
+    }):[]
+  };
+}
@@ -435 +462,0 @@
-  const nodes=result?.structuredContent?.content_refs;
@@ -438,7 +465,2 @@
-  const roles=["heading","button","statictext","textbox"];
-  const labels=["Username","Password","Authenticator or recovery code","Sign in",
-    "Local demo","Protected application access",
-    "Signed in. Protected application access is available."];
-  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
-    labels.includes(spec.expected_label)&&
-    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
+  return projectPublicDecision(result).observed.some(n=>
+    n.role===spec.expected_role&&n.name===spec.expected_label);
@@ -478,0 +501 @@
+    public_decision:own.public_decision??null,
```

### Exact inverse

```diff
--- a/archived-d01-continuation-cell.js
+++ b/archived-d01-continuation-cell.js
@@ -329 +328,0 @@
-  own.public_decision=projectPublicDecision(result);
@@ -435,27 +433,0 @@
-function projectPublicDecision(result) {
-  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
-  const labels=[
-    ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
-    ["heading","Local demo wants to use your riAuth account"],
-    ["heading","Continue to Local demo?"],["heading","Protected application access"],
-    ["button","Sign in"],["button","Allow"],["button","Continue"],
-    ["textbox","Username"],["textbox","Password"],
-    ["textbox","Authenticator or recovery code (if enabled)"],
-    ["statictext","Signed in. Protected application access is available."]
-  ];
-  const valid=result?.isError!==true&&s?.status==="ok"&&s?.snapshot?.complete===true&&
-    !nodes.some(n=>n.name==="Local demo could not complete this request.");
-  return {
-    observed:valid?labels.filter(([role,name])=>
-      nodes.some(n=>n.role===role&&n.name===name)).map(([role,name])=>({role,name})):[],
-    refs:valid&&Array.isArray(s?.refs)?s.refs.flatMap(n=>{
-      const pair=labels.find(([role,name])=>n?.role===role&&n.name===name),ref=n?.ref;
-      if(!pair||typeof ref!=="string"||!/^p[0-9]+:[0-9]+$/.test(ref))return [];
-      const [role,name]=pair;
-      const action=role==="button"?"click":
-        role==="textbox"&&["Username","Password"].includes(name)?"type":null;
-      if(action===null||!Array.isArray(n.actions)||!n.actions.includes(action))return [];
-      return [{role,name,action,ref}];
-    }):[]
-  };
-}
@@ -462,0 +435 @@
+  const nodes=result?.structuredContent?.content_refs;
@@ -465,2 +438,7 @@
-  return projectPublicDecision(result).observed.some(n=>
-    n.role===spec.expected_role&&n.name===spec.expected_label);
+  const roles=["heading","button","statictext","textbox"];
+  const labels=["Username","Password","Authenticator or recovery code","Sign in",
+    "Local demo","Protected application access",
+    "Signed in. Protected application access is available."];
+  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
+    labels.includes(spec.expected_label)&&
+    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
@@ -501 +478,0 @@
-    public_decision:own.public_decision??null,
```

## Complete readable candidate — ARCHIVED ONLY, UNEXECUTED

```javascript
// Prospective ONE functions.exec cell; NOT executed in this design phase.
const CLOCK="import json,time\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\n";
const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
const READBACK="import json,os,pathlib,stat,subprocess,sys,time\nc=json.loads(sys.argv[1])\nchildren=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'\ndef finite_observation(value):\n    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:\n        return None\n    diagnostic=value['diagnostic']\n    if diagnostic is None:\n        return {'observed':value['observed'],'diagnostic':None}\n    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:\n        return None\n    sites=('handler','server','main')\n    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')\n    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')\n    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))\n    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:\n        return None\n    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):\n        return None\n    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)\n    try:\n        info=os.fstat(fd)\n        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):\n            raise ValueError('fixed_outer_evidence_invalid')\n        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)\n        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')\n        data=json.loads(raw_bytes)\n    finally:os.close(fd)\n    outer_observation=finite_observation(data.get('unexpected_failure_observation'))\n    projection=data.get('unexpected_observation_projection')\n    if projection not in ('unavailable','valid','invalid'):projection='invalid'\n    if projection=='valid' and outer_observation is None:projection='invalid'\n    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0\n    if started:allowed.add('helper')\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\n        helper_pid=data['helper_pid'] if started else None\npids=list(c['owned_pids'])\nif helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))\n";
const MARKER="import os,pathlib,stat,sys\nlab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])\nif guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')\ninfo=lab.lstat()\nif not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')\nos.kill(guard,0);os.kill(server,0)\nif (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')\nfd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nos.close(fd)\nprint('{\"prepared_marker_written\":true}')\n";
const PERSIST="import json,os,pathlib,sys\npath=pathlib.Path(sys.argv[1])\nrecord=json.loads(sys.argv[2])\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\ndef clocks(value):\n    if isinstance(value,dict):\n        for k,v in list(value.items()):\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\n                value[k]=int(v)\n            else:clocks(v)\n    elif isinstance(value,list):\n        for v in value:clocks(v)\nclocks(record)\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nwith os.fdopen(fd,'w',encoding='ascii') as f:\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\n');f.flush();os.fsync(f.fileno())\nprint(json.dumps({'written_exclusive':True}))\n";
const own=load("d01_immediate_owned_handles");
const OBSKEY="d01_immediate_observation_record";
if (!own || own.runtime_release!==true || own.driver_owned!==true ||
    own.exact_bound!==true || own.single_controller!==true ||
    !["prepared_entry","browser_decision"].includes(own.phase) ||
    (own.phase==="prepared_entry"&&(own.prepared_gate!==true||own.helper_pid!==null||
      own.fixture_ready===true||own.listener_pid_proof===true)) ||
    (own.phase==="browser_decision"&&(own.fixture_ready!==true||own.listener_pid_proof!==true)) ||
    own.fresh_paths_preflight!==true ||
    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!=="string" ||
    new Set([own.guard_pid,own.server_pid,own.browser_pid,
       ...(own.helper_pid===null?[]:[own.helper_pid])]).size!==(own.helper_pid===null?3:4) ||
    ![own.guard_pid,own.server_pid,own.browser_pid,own.exec_session,
       ...(own.helper_pid===null?[]:[own.helper_pid])]
       .every(v=>Number.isSafeInteger(v)&&v>0) ||
    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,
       own.event_out,own.cleanup_out].every(v=>typeof v==="string"&&v.length>0)) {
  text({proposal_refused:"fresh_owned_context_required"});exit();
}
const prior=load(OBSKEY);
if(own.closed===true||prior?.cleanup_started===true){
  text({proposal_refused:"fixture_already_stopped",resource_release_proven:false});exit();
}
const record=prior??{
  schema:"riauth.d01-immediate-observation-cleanup/v1",
  first_failure:null,first_observation_wall_ms:null,
  first_event_proven:false,whole_cleanup_within60_proven:false,
  unexpected_failure_observation:null,diagnostic_errors:[],cleanup_started:false,
  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],
  final_absence:null,owned_child_exits:null,controller_exit:null,
  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null
};
const sq=s=>"'"+String(s).replace(/'/g,"'\\''")+"'";
const retain=()=>store(OBSKEY,record);
const cleanupError=label=>{
  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);
  retain();
};
const local=async(source,arg)=>{
  const cmd="python3 -c "+sq(source)+(arg===undefined?"":" "+sq(JSON.stringify(arg)));
  const r=await tools.exec_command({cmd,workdir:own.workspace,
    yield_time_ms:10000,max_output_tokens:2000});
  record.local_tool_receipts.push({
    label:source===CLOCK?"clock":source===STOP?"stop":source===READBACK?"readback":"unknown",
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    session_id:typeof r.session_id==="number"?r.session_id:null
  });retain(); // Numeric collector receipt BEFORE comparisons.
  if(r.session_id!==undefined) {
    cleanupError("owned_local_command_unjoined");throw new Error("local_pending");
  }
  if(r.exit_code!==0)throw new Error("local_failed");
  return JSON.parse(r.output);
};
function finiteObservation(value) {
  const exact=(v,keys)=>v!==null&&typeof v==="object"&&!Array.isArray(v)&&
    Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
  if(!exact(value,["observed","diagnostic"])||typeof value.observed!=="boolean")return null;
  const d=value.diagnostic;
  if(d===null)return {observed:value.observed,diagnostic:null};
  if(!value.observed||!exact(d,["site","exception_class","own_function","own_line"])||
     !["handler","server","main"].includes(d.site)||
     !["AttributeError","TypeError","ValueError","KeyError","OSError","BrokenPipeError",
       "ConnectionResetError","TimeoutError","other"].includes(d.exception_class))return null;
  const functions=["HeaderReader.readline","DemoServer.process_request","Demo.begin","Demo.callback",
    "Demo.invoke","Handler.handle_one_request","Handler.send_error","Handler.get","Handler.reply","main"];
  if(!((d.own_function===null&&d.own_line===null)||
       (functions.includes(d.own_function)&&Number.isSafeInteger(d.own_line)&&
        d.own_line>=1&&d.own_line<=1024)))return null;
  return {observed:true,diagnostic:{site:d.site,exception_class:d.exception_class,
    own_function:d.own_function,own_line:d.own_line}};
}
function diagnostic(value,projection) {
  const projected=finiteObservation(value);
  if(projection==="invalid"||!["valid","unavailable"].includes(projection)||
     (projection==="valid"&&projected===null)) {
    if(!record.diagnostic_errors.includes("observer_projection_invalid"))
      record.diagnostic_errors.push("observer_projection_invalid");
  }
  if(projected!==null&&record.unexpected_failure_observation===null)
    record.unexpected_failure_observation=projected;
  retain(); // No raw diagnostic fallback and no first-failure or outcome mutation.
}
const ns=c=>BigInt(c.monotonic_ns);
const getClock=()=>local(CLOCK);
let controllerJoined=false;
let controllerPollAvailable=true;
let controllerBuffer="";
let cleanupStarted=false;
let lastSnapshotFlags=null;
function latch(label,receivedWall) {
  if(record.first_failure===null) {
    record.first_failure=label;
    record.first_observation_wall_ms=receivedWall;
    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.
  }
}
function observeController(r,receivedWall) {
  const observation={received_wall_ms:receivedWall,
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    helper_completed:false,helper_exit:null,fixture_finished:false};
  // Complete numeric result projection is retained BEFORE comparisons.
  record.controller_observations.push(observation);retain();
  if(typeof r.exit_code==="number") {
    controllerJoined=true;record.controller_exit=r.exit_code;retain();
  }
  controllerBuffer+=typeof r.output==="string"?r.output:"";
  if(controllerBuffer.length>16384) {
    latch("controller_observation_invalid",receivedWall);controllerBuffer="";return;
  }
  let cut;
  while((cut=controllerBuffer.indexOf("\n"))>=0) {
    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch{
      latch("controller_observation_invalid",receivedWall);continue;
    }
    if(Object.hasOwn(event,"unexpected_failure_observation"))
      diagnostic(event.unexpected_failure_observation,event.unexpected_observation_projection);
    if(event.fixture_ready===true) {
      if(event.guard_pid!==own.guard_pid||event.server_pid!==own.server_pid||
         event.lab!==own.lab||!Number.isSafeInteger(event.helper_pid)||event.helper_pid<=0||
         [own.guard_pid,own.server_pid,own.browser_pid].includes(event.helper_pid)||
         (own.helper_pid!==null&&own.helper_pid!==event.helper_pid))
        latch("fixture_ready_unconfirmed",receivedWall);
      else {
        own.helper_pid=event.helper_pid;own.fixture_ready=true;own.listener_pid_proof=true;
        store("d01_immediate_owned_handles",own);
      }
    }
    if(event.helper_completed===true) {
      observation.helper_completed=true;
      observation.helper_exit=typeof event.exit==="number"?event.exit:null;
      retain();
      if(event.exit!==0)latch("helper_failed",receivedWall);
      else if(!cleanupStarted&&record.protected_after_page!==true&&
              !(own.phase==="browser_decision"&&
                own.next_decision?.kind==="protected_after"))
        latch("helper_completed_before_app_checkpoint",receivedWall);
    }
    if(event.fixture_finished===true) {
      observation.fixture_finished=true;retain();
      if(!cleanupStarted)latch("controller_completed_before_app_checkpoint",receivedWall);
    }
  }
  if(controllerJoined&&!cleanupStarted)
    latch("controller_completed_before_app_checkpoint",receivedWall);
}
async function pollController() {
  if(controllerJoined||!controllerPollAvailable)return;
  try {
    const r=await tools.write_stdin({session_id:own.exec_session,
      chars:"",yield_time_ms:5000,max_output_tokens:2000});
    const receivedWall=Date.now();
    observeController(r,receivedWall);
  } catch {
    controllerPollAvailable=false;
    latch("controller_observation_unavailable",Date.now());
    if(cleanupStarted)cleanupError("controller_join_unconfirmed");
  }
}
async function timedDriver(label,allocation,operation,project) {
  let start=null,end=null,result=null,state="unknown";
  try{start=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,
    result_state:"unknown",elapsed_ms:null,over_budget:null};
  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.
  try {
    result=await operation();
    state=result?.isError===true?"refused":"returned";
  } catch {state="exception";}
  try{end=await getClock();}catch{cleanupError("clock_unavailable");}
  action.end_clock=end;action.result_state=state;
  retain(); // End/result retained BEFORE budget comparison.
  if(start&&end) {
    const d=ns(end)-ns(start);
    if(d>=0n) {
      action.elapsed_ms=Number(d/1000000n);
      action.over_budget=action.elapsed_ms>allocation;
    } else cleanupError("clock_invalid");
  }
  if(action.over_budget)cleanupError("driver_operation_over_budget");
  if(state!=="returned")cleanupError("driver_operation_unconfirmed");
  if(project)project(result,state);
  retain();
}
async function cleanup() {
  store("d01_fresh_password_input",null); // Clear before any cleanup await.
  if(cleanupStarted)return;
  cleanupStarted=true;record.cleanup_started=true;retain();
  try {
    const r=await local(STOP,{
      lab:own.lab,event_out:own.event_out,
      first_failure:record.first_failure,
      first_observation_wall_ms:record.first_observation_wall_ms
    });
    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();
    if(r.event_state!=="written")cleanupError("observation_metadata_write_unconfirmed");
    if(r.marker_state==="stop_unconfirmed")cleanupError("stop_unconfirmed");
    if(r.marker_state==="ownership_unknown")cleanupError("ownership_unknown");
  } catch {cleanupError("stop_or_clock_record_unavailable");}
  // No outstanding Driver call exists here: all page calls above were awaited.
  // Original stop protocol already causes the controller's owned-child finally.
  await timedDriver("kill_app",30000,
    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));
  await timedDriver("end_session",15000,
    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{
      record.session_ended=state==="returned"&&
        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;
      retain();
    });
  await timedDriver("list_windows",5000,
    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{
      const windows=r?.structuredContent?.windows;
      record.window_count=state==="returned"&&Array.isArray(windows)?windows.length:null;
      retain();
    });
  let readStart=null;
  try{readStart=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label:"owned_join_readback",start_clock:readStart,end_clock:null,
    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:"unknown"};
  record.actions.push(action);retain();
  if(readStart&&record.first_clock) {
    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));
    retain();
  }
  // Continue essential join after60s if late; never claim that deadline enforced.
  // Do not poll another exec session or send a manual process signal.
  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {
    await pollController();
    if(record.first_clock) {
      try {
        const c=await getClock();record.last_join_clock=c;retain();
        if(ns(c)-ns(record.first_clock)>60000000000n)
          cleanupError("cleanup_budget_exceeded");
      } catch{cleanupError("clock_unavailable");}
    }
  }
  if(!controllerJoined)cleanupError("child_reap_incomplete");
  try {
    const r=await local(READBACK,{
      owned_pids:[own.guard_pid,own.server_pid,own.browser_pid,
        ...(own.helper_pid===null?[]:[own.helper_pid])],
      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid
    });
    action.end_clock=r.clock;action.result_state="returned";
    record.owned_child_exits=r.owned_child_exits;
    diagnostic(r.unexpected_failure_observation,r.unexpected_observation_projection);
    if(Number.isSafeInteger(r.helper_pid)&&r.helper_pid>0)own.helper_pid=r.helper_pid;
    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,
      lab:r.lab_absent,window_count:record.window_count??null,
      session_ended:record.session_ended===true};
    record.first_observation_to_final_wall_ms=record.first_observation_wall_ms===null?null:
      Date.now()-record.first_observation_wall_ms;
    retain(); // Full fixed readback/exits/clock BEFORE comparisons.
    if(readStart) {
      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);
      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;
    }
    if(record.first_clock)
      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);
    if(action.over_budget)cleanupError("cleanup_budget_exceeded");
    const a=record.final_absence;
    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||
       record.owned_child_exits.length!==(own.helper_pid===null?6:7))
      cleanupError("child_reap_incomplete");
    if(!Object.values(a.owned_pids).every(v=>v===true)||
       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||
       a.window_count!==0||a.session_ended!==true)
      cleanupError("owned_resource_absence_unproven");
  } catch {action.result_state="exception";cleanupError("owned_readback_unavailable");}
  cleanupError("first_event_unproven");
  record.whole_cleanup_within60_proven=false;retain();
  // Exclusive new file only; existing outer/helper/provider evidence untouched.
  try {
    const cmd="python3 -c "+sq(PERSIST)+" "+sq(own.cleanup_out)+" "+sq(JSON.stringify(record));
    const r=await tools.exec_command({cmd,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500});
    record.local_tool_receipts.push({label:"persist",
      exit:typeof r.exit_code==="number"?r.exit_code:null,
      session_id:typeof r.session_id==="number"?r.session_id:null});retain();
    if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
    if(r.session_id!==undefined||r.exit_code!==0)
      cleanupError("cleanup_metadata_write_unconfirmed");
  } catch {cleanupError("cleanup_metadata_write_unconfirmed");}
  controllerBuffer="";own.closed=true;store("d01_immediate_owned_handles",own);
}
async function checked(label,operation,predicate) {
  if(record.first_failure!==null)return null;
  if(Date.now()>=own.start_epoch_ms+840000) {
    latch("browser_active_deadline",Date.now());await cleanup();return null;
  }
  let result,receivedWall;
  try {
    result=await operation();receivedWall=Date.now();
    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});retain();
    if(result?.isError===true)latch("browser_tool_refused",receivedWall);
    else if(!predicate(result))latch("browser_decision_unconfirmed",receivedWall);
  } catch {latch("browser_tool_or_decision_exception",Date.now());}
  if(record.first_failure!==null)await cleanup();
  return record.first_failure===null?result:null;
}
const snapshotArgs={session:own.session,target_id:own.target_id,tab_id:own.tab_id,
  snapshot_format:"semantic_v2",include_screenshot:false};
function page(result,kind) {
  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
  const flags={
    status_ok:result?.isError!==true&&s?.status==="ok",
    complete:s?.snapshot?.complete===true,
    heading_match:nodes.some(n=>n.role==="heading"&&n.name===
      (kind==="protected_after"?"Protected application access":"Local demo")),
    error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
    required_text:nodes.some(n=>n.name===(kind==="protected_before"?"Sign in required.":
      kind==="protected_after"?"Signed in. Protected application access is available.":
      "Use Sign in to open this local application.")),
    interactive_ref_present:Array.isArray(s?.refs)&&
      s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
  };
  record.last_page_flags={kind,...flags};retain();
  // Only provider refs and fixed public-label matches survive this raw snapshot.
  own.fresh_refs=Array.isArray(s?.refs)?s.refs.filter(n=>typeof n.ref==="string"&&
    /^p[0-9]+:[0-9]+$/.test(n.ref)).map(n=>n.ref):[];
  own.public_decision=projectPublicDecision(result);
  store("d01_immediate_owned_handles",own);
  if(flags.error_match)return false;
  if(!flags.status_ok||!flags.complete)return false;
  if(kind==="generic_checked") {
    if(nodes.some(n=>n.role==="heading"&&n.name==="Protected application access")&&
       nodes.some(n=>n.name==="Signed in. Protected application access is available."))
      record.protected_after_page=true;
    return true;
  }
  const ok=flags.heading_match&&flags.required_text&&
    (kind!=="application"||flags.interactive_ref_present);
  if(ok&&kind==="protected_after")record.protected_after_page=true;
  return ok;
}
async function navigateAndSnapshot(url,kind) {
  if(await checked("browser_navigation",
      ()=>tools.mcp__cua_driver__browser_navigate({session:own.session,
        target_id:own.target_id,tab_id:own.tab_id,url}),
      r=>r?.isError!==true)) {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,kind));
  }
}
async function entry() {
  // The exact Driver-owned blank handles already exist while the180s gate waits.
  // There is no model yield, tool-description load or rebind after this marker.
  const markerCommand="python3 -c "+sq(MARKER)+" "+sq(own.lab)+" "+
    sq(own.guard_pid)+" "+sq(own.server_pid);
  await checked("prepared_marker",
    ()=>tools.exec_command({cmd:markerCommand,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500}),r=>{
      record.local_tool_receipts.push({label:"marker",
        exit:typeof r.exit_code==="number"?r.exit_code:null,
        session_id:typeof r.session_id==="number"?r.session_id:null});retain();
      if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
      return r.exit_code===0&&r.session_id===undefined&&
        JSON.parse(r.output).prepared_marker_written===true;
    });
  const readyDeadline=Date.now()+30000;
  while(record.first_failure===null&&own.fixture_ready!==true&&Date.now()<readyDeadline)
    await pollController();
  if(record.first_failure===null&&own.fixture_ready!==true)
    latch("fixture_ready_unconfirmed",Date.now());
  if(record.first_failure!==null){await cleanup();return;}
  await pollController(); // ONE immediate pre-navigation poll, no RP HTTP probe.
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/protected","protected_before");
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/","application");
  if(record.first_failure===null)await pollController(); // ONE post-entry poll.
  if(record.first_failure===null) {
    own.phase="browser_decision";
    store("d01_immediate_owned_handles",own);
  }
  if(record.first_failure!==null)await cleanup();
}
async function continuation() {
  const spec=own.next_decision;
  const allowedKinds=["click","username","password","snapshot","protected_after"];
  if(!spec||!allowedKinds.includes(spec.kind)) {
    latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
  }
  await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  if(spec.kind==="protected_after") {
    // Read the fresh callback-following protected page; do not send another RP request.
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,"protected_after"));
  } else if(spec.kind==="snapshot") {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
      r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  } else {
    if(typeof spec.ref!=="string"||!Array.isArray(own.fresh_refs)||
       !own.fresh_refs.includes(spec.ref)) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    own.fresh_refs=[];store("d01_immediate_owned_handles",own); // Single-use snapshot ref.
    const args={session:own.session,target_id:own.target_id,tab_id:own.tab_id,ref:spec.ref};
    const operation=spec.kind==="click"?
      ()=>tools.mcp__cua_driver__browser_click({...args,input_route:"dom_event"}):
      ()=>tools.mcp__cua_driver__browser_type({...args,replace:true,
        text:spec.kind==="username"?"admin":load("d01_fresh_password_input")});
    if(spec.kind==="password"&&(typeof load("d01_fresh_password_input")!=="string"||
       !/^[A-Za-z0-9_-]{30,128}$/.test(load("d01_fresh_password_input")))) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    await checked("browser_input",operation,r=>{
      const s=r?.structuredContent;
      return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
    }); // Dispatch alone never earns an application outcome or journey credit.
    if(spec.kind==="password")store("d01_fresh_password_input",null);
    if(record.first_failure===null)
      await checked("browser_snapshot",
        ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
        r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  }
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null)await cleanup();
  else if(spec.kind==="protected_after") {
    record.cleanup_requested_wall_ms=Date.now();retain();
    await cleanup();
  }
}
function projectPublicDecision(result) {
  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
  const labels=[
    ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
    ["heading","Local demo wants to use your riAuth account"],
    ["heading","Continue to Local demo?"],["heading","Protected application access"],
    ["button","Sign in"],["button","Allow"],["button","Continue"],
    ["textbox","Username"],["textbox","Password"],
    ["textbox","Authenticator or recovery code (if enabled)"],
    ["statictext","Signed in. Protected application access is available."]
  ];
  const valid=result?.isError!==true&&s?.status==="ok"&&s?.snapshot?.complete===true&&
    !nodes.some(n=>n.name==="Local demo could not complete this request.");
  return {
    observed:valid?labels.filter(([role,name])=>
      nodes.some(n=>n.role===role&&n.name===name)).map(([role,name])=>({role,name})):[],
    refs:valid&&Array.isArray(s?.refs)?s.refs.flatMap(n=>{
      const pair=labels.find(([role,name])=>n?.role===role&&n.name===name),ref=n?.ref;
      if(!pair||typeof ref!=="string"||!/^p[0-9]+:[0-9]+$/.test(ref))return [];
      const [role,name]=pair;
      const action=role==="button"?"click":
        role==="textbox"&&["Username","Password"].includes(name)?"type":null;
      if(action===null||!Array.isArray(n.actions)||!n.actions.includes(action))return [];
      return [{role,name,action,ref}];
    }):[]
  };
}
function publicPredicate(result,spec) {
  // Root must pin one printed public expected label/role before that action.
  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.
  return projectPublicDecision(result).observed.some(n=>
    n.role===spec.expected_role&&n.name===spec.expected_label);
}
try {
  if(own.phase==="prepared_entry")await entry();else await continuation();
  // Do not carry unvalidated partial controller text across this cell boundary.
  while(controllerBuffer.length>0&&record.first_failure===null) {
    const beforePollWall=Date.now();
    if(beforePollWall>=own.start_epoch_ms+840000) {
      latch("browser_active_deadline",beforePollWall);break;
    }
    if(controllerJoined||!controllerPollAvailable) {
      latch("controller_observation_invalid",beforePollWall);break;
    }
    await pollController(); // Same owned session; observer retains receipt first.
    const afterPollWall=Date.now();
    if(afterPollWall>=own.start_epoch_ms+840000)
      latch("browser_active_deadline",afterPollWall);
  }
  if(record.first_failure!==null)await cleanup();
} catch {
  latch("composed_decision_exception",Date.now());
  await cleanup();
}
if(record.first_failure!==null) {
  text({result:"failed",first_failure:record.first_failure,
    unexpected_failure_observation:record.unexpected_failure_observation,
    diagnostic_errors:record.diagnostic_errors,cleanup_errors:record.cleanup_errors,
    final_absence:record.final_absence,controller_exit:record.controller_exit,
    whole_cleanup_within60_proven:false,
    resource_release_proven:record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v))});
} else {
  text({result:"browser_decision_observed_only",page_flags:record.last_page_flags??null,
    public_decision:own.public_decision??null,
    journey_credit:false,cleanup_errors:record.cleanup_errors,
    resource_release_proven:record.cleanup_started===true&&record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v)),
    whole_cleanup_within60_proven:false});
}
```

## Whole-byte and static AST preservation proof

Three unique old/new textual anchors reverse the entire candidate to the exact 32502-byte 9f source. Independent zero-context diff application in both directions also reconstructs the complete original/candidate without relying on those string replacements. No other candidate span changes. The structural AST inverse removes only the new projector, removes the added page assignment, restores only the old publicPredicate node and removes only the successful-output property; full normalized AST equality then holds. No collector, candidate/case/cell/VM/observer/helper/controller/main/library/import or runtime path is evaluated by this checker. Only the installed Node bundled Acorn parser and static source/data comparison run. Python AST parsing also passes for all five literal collector sources, the exact controller Python body and fixed descriptor helper; none is imported.

Normalized baseline AST SHA-256 `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74`; candidate `cce3210177ab2e377a58153c62a2c50f047b031f6e979e0ac17d22b4a46235ce`; inverse `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74`. New projector function SHA-256 `414fa7865ac064f9c907af5badf4294ebaa43df319ad56facccd6033b7047feb`.

| Existing function | Byte equality | 9f body SHA-256 | Proposed body SHA-256 |
| --- | --- | --- | --- |
| finiteObservation | Exact | `b57854dabbc985407451ab0f90272a0250218156ea298750b5550951193b19c1` | `b57854dabbc985407451ab0f90272a0250218156ea298750b5550951193b19c1` |
| diagnostic | Exact | `08fb60069a6523d07783754388cf2479d3a2895a1607ec87fb80101b136d0af1` | `08fb60069a6523d07783754388cf2479d3a2895a1607ec87fb80101b136d0af1` |
| latch | Exact | `6cc3aae70510550f21304ceb79587d03c800cb459d50c1bd714ea77f4541199c` | `6cc3aae70510550f21304ceb79587d03c800cb459d50c1bd714ea77f4541199c` |
| observeController | Exact | `1e8968dd48f70ab7c6b88fbb250e1e4ba57879604a596a8d901a1f0bd635cb16` | `1e8968dd48f70ab7c6b88fbb250e1e4ba57879604a596a8d901a1f0bd635cb16` |
| pollController | Exact | `6dfcfa6d89bfc4345ca77a8304bd8390353fb8a3e80285c9c66e7187ff2d7d4c` | `6dfcfa6d89bfc4345ca77a8304bd8390353fb8a3e80285c9c66e7187ff2d7d4c` |
| timedDriver | Exact | `f7c305d432a3a11885b71035e771ca0567520165fbe697f74f2d65a2d981bd0b` | `f7c305d432a3a11885b71035e771ca0567520165fbe697f74f2d65a2d981bd0b` |
| cleanup | Exact | `0d8dc617795d72e24519b956f324cf08145838e157840418f107e8436457ffca` | `0d8dc617795d72e24519b956f324cf08145838e157840418f107e8436457ffca` |
| checked | Exact | `df302d63be0451bcb392a0d6ab733f7d9d552a5605a82e99375bb6ac8b8cc1a9` | `df302d63be0451bcb392a0d6ab733f7d9d552a5605a82e99375bb6ac8b8cc1a9` |
| page | Declared projection seam only | `d4aa96f964c11de6a2d8a15dbfe965d69edf40a0e3bf38d9c9cc614dcdafd88c` | `c5ea57a4ffee712c4128a0e2e8431774a45eb59eabafa4e62a2865f9d6da58f6` |
| navigateAndSnapshot | Exact | `0f2337a9f2c8ba20c34be23f1249d4352f21267ea8f45b475a2ce72d60b11a65` | `0f2337a9f2c8ba20c34be23f1249d4352f21267ea8f45b475a2ce72d60b11a65` |
| entry | Exact | `6b6350f4abd79a799c09fa4c1326a45fcdf824818d221aba0e790608e0902118` | `6b6350f4abd79a799c09fa4c1326a45fcdf824818d221aba0e790608e0902118` |
| continuation | Exact | `04365b46726732d62762a351c82df05cb7354b2aefb65600674898068410c0cb` | `04365b46726732d62762a351c82df05cb7354b2aefb65600674898068410c0cb` |
| publicPredicate | Declared projection seam only | `5f4f55c95ae0d062c85892ba92a09ae8263d981f63fe8fd4bcc81da1c29778a5` | `2edfc621bda88598e8a86b28b29a9ea6473cf4546f2e0d6164a712f80ccee8aa` |

Eleven of thirteen original complete function spans are byte-identical; only page/publicPredicate change. The fourteenth function is the pure projector. The successful output property is the sole other top-level AST edit. The five complete collector declarations remain byte-identical, with these unescaped literal-source identities:

| Collector | Bytes | SHA-256 |
| --- | ---: | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

### Complete parser-only checker

3681 bytes / 54 lines / SHA-256 `b683ad96028be0619545004f1970c96da86282e333f3d002e26aba3035b37c68`

```javascript
const fs=require("node:fs"),crypto=require("node:crypto");
const acorn=require("internal/deps/acorn/acorn/dist/acorn");
const input=JSON.parse(fs.readFileSync(0,"utf8"));
const parse=s=>acorn.parse(s,{ecmaVersion:"latest",sourceType:"module"});
const normalize=v=>{
  if(typeof v==="bigint")return {ast_bigint_decimal:v.toString(10)};
  if(Array.isArray(v))return v.map(normalize);
  if(v&&typeof v==="object")return Object.fromEntries(Object.entries(v)
    .filter(([k])=>!["start","end","loc","range"].includes(k)).map(([k,x])=>[k,normalize(x)]));
  return v;
};
const key=v=>JSON.stringify(normalize(v));
const hash=s=>crypto.createHash("sha256").update(s).digest("hex");
const before=parse(input.base),after=parse(input.candidate),restored=structuredClone(normalize(after));
const fn=(tree,name)=>tree.body.find(n=>n.type==="FunctionDeclaration"&&n.id.name===name);
const projection=fn(after,"projectPublicDecision");
if(!projection||after.body.filter(n=>n.type==="FunctionDeclaration").length!==14)throw Error("function_count");
restored.body=restored.body.filter(n=>!(n.type==="FunctionDeclaration"&&n.id.name==="projectPublicDecision"));
const page=fn(restored,"page");
const assignment=normalize(parse('own.public_decision=projectPublicDecision(result);').body[0]);
const pageIndex=page.body.body.findIndex(n=>key(n)===key(assignment));
if(pageIndex!==5)throw Error("page_assignment_position");
page.body.body.splice(pageIndex,1);
const predIndex=restored.body.findIndex(n=>n.type==="FunctionDeclaration"&&n.id.name==="publicPredicate");
restored.body[predIndex]=normalize(fn(before,"publicPredicate"));
const final=restored.body.at(-1);
if(final.type!=="IfStatement")throw Error("final_output_shape");
const output=final.alternate.body[0].expression.arguments[0];
const extra=output.properties.findIndex(n=>n.key.name==="public_decision");
if(extra!==2||output.properties[extra].value.type!=="LogicalExpression"||
   output.properties[extra].value.operator!=="??")throw Error("output_property");
output.properties.splice(extra,1);
if(key(restored)!==key(before))throw Error("whole_ast_inverse");
const functions=before.body.filter(n=>n.type==="FunctionDeclaration").map(n=>{
  const c=fn(after,n.id.name),b=input.base.slice(n.start,n.end),a=input.candidate.slice(c.start,c.end);
  return {name:n.id.name,base_sha256:hash(b),candidate_sha256:hash(a),bytes_equal:b===a};
});
if(functions.filter(n=>n.bytes_equal).length!==11||
   functions.filter(n=>!n.bytes_equal).map(n=>n.name).join(",")!=="page,publicPredicate")throw Error("function_scope");
const names=["CLOCK","STOP","READBACK","MARKER","PERSIST"];
const collectors=names.map(name=>{
  const decl=t=>t.body.find(n=>n.type==="VariableDeclaration"&&n.declarations.some(d=>d.id.name===name));
  const b=decl(before),c=decl(after);
  if(input.base.slice(b.start,b.end)!==input.candidate.slice(c.start,c.end))throw Error("collector_changed");
  const source=b.declarations[0].init.value;
  return {name,bytes:Buffer.byteLength(source),sha256:hash(source),source};
});
const labels=projection.body.body.find(n=>n.type==="VariableDeclaration"&&n.declarations[0].id.name==="labels")
  .declarations[0].init.elements.map(n=>n.elements.map(v=>v.value));
if(labels.length!==12||new Set(labels.map(n=>JSON.stringify(n))).size!==12)throw Error("fixed_labels");
console.log(JSON.stringify({parser_only:true,candidate_evaluated:false,whole_ast_inverse:true,
  base_normalized_ast_sha256:hash(key(before)),candidate_normalized_ast_sha256:hash(key(after)),
  restored_normalized_ast_sha256:hash(key(restored)),functions,collectors,labels,
  projection_function_sha256:hash(input.candidate.slice(projection.start,projection.end))}));
```

The checker accepts only the baseline/candidate source JSON through stdin, calls Acorn parse, manipulates AST/data and compares hashes. It contains no VM/eval/Function constructor or execution of source under review. The installed Node path was selected by metadata (`command -v node`); no version/native/product probe or tool installation was run. Source strings were assembled in memory and serialized only as this Markdown archive.

## Actual static errors, boundaries and next review

Two initial source-fence extraction reads exited `1` because the local text extractor requested capture group 2 from a one-group regex; the corrected source-only extractor used group 1 and then read the complete pinned cell/controller successfully. One initial selected Git search exited with zsh `no matches found: src/oauth*`; the literal-path `-- src` search succeeded. These are retained local static-read errors, not helper/product/provider/fixture failures. No source under review ran during those failures or their corrections.

Actual source/design checks passed: exact immutable sizes/hashes; complete candidate/controller source extraction and canonical delimiter rule; three-anchor whole-byte inverse; independent forward/inverse diff reconstruction; Acorn parse/normalized AST reversal and all function/collector identities (numeric exit `0`); Python AST parse only for controller, helper and five collectors. No memory stub/case/envelope ran. This new projection needs root full review, independent source review and a separately reviewed changed-candidate memory validation before any real RiWork Cua.ai Driver-only fixture. The separately owned 9f memory envelope and all old results remain dated; no ownership is duplicated or endorsement inferred. No real browser/helper/Cargo/native/provider/HTTP/CLI/socket/PG service/network/query/download/dispatch/capacity/signal invocation or slot acquisition/release occurred.

All collectors, first-failure/receipt order, helper-zero final-read allowance, partial-line drain, password clear/latch/input wrappers, 840-second active/900-second whole budgets, essential join/late cleanup/nonrenewed60s, private 0700/0600 output, retained outcome flags and journey_credit false remain unchanged. No candidate source materialization, helper/controller/product/verifier/HTML/guide/workflow/test edit or alignment occurs. Original D01/D05 remain open; primary42 and DONE rows are unchanged. Root owns any further reservation, publication, actual next decision and gate status. Source omission is not a retrospective diagnosis; cause/sender/lost values remain UNKNOWN.

Actual report-aware checks passed: `python3 scripts/check-docs.py` exited `0` (`Markdown links and build-directory layout checked`), `git diff --check` exited `0`, final LF/trailing-whitespace and mode `0644` checks passed, and source/CLI/D01 reports remain byte-identical to entry. The only untracked path is this new report; no tracked working/index mutation exists. Witness-line review corrected the draft consent-handler, app-name and RP HTML ranges to the actual immutable source lines; candidate behavior was unaffected. Final serialized-fence/diff/hash reconstruction, staged new-report-only scope/equality/whitespace and clean post-commit checks accompany the immutable handoff. No runtime slot was acquired or released.


## Source-only public-projection memory design, 2026-10-03

Reservation: wave30_D01_public_projection_memory_design, project
891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT
f2e8500e-2e56-47e3-b60e-9f81bbc8cff2. Entry is
99d740c126cfe1a28b28d629c026e87ddb3aba46. APPEND ONLY. The complete48288
report prefix remains60522bytes/SHA256
16f0e6c4b77469d3057489e73bb1bea93cfe2668b76171c244594827a75a8321,
including its full33701-byte508-line candidate and all historical evidence.

This is a complete proposed memory payload, not a run or a source correction.
It executes the exact complete33701-byte candidate only in a FUTURE separately
released bounded envelope. No candidate span is edited, selected projector
function copied into a test implementation, or mocked replacement inserted
into that cell. Expected observations/action associations are explicit fixture
contract data; the consumer under test is the full cell, including page,
publicPredicate, successful output, retained store, input/ref guard, first
failure and essential synthetic cleanup.

Read the full meaningful old25-case payload logic and source/data bindings at
36ac893435b3aa05556d43e8576851ced20c176e:
docs/roadmap/local-wave30-d01-continuation-correction-plan.md, including its
476-line readable logic,25 named cases, controlled transport, packet protocol
and complete historical controller logic outside payload data. Neither the
old payload nor its functions/imports/VM/cases/controller executed here.
Old25 remains UNRUN; this design cannot credit any past memory result or
real journey to it.

| Archived source unit | Bytes / lines | SHA256 |
| --- | --- | --- |
| Whole9f baseline, also old36ac candidate | 32502 /485 | 7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8 |
| Exact complete projection candidate, readable in preserved prefix | 33701 /508 | 505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f |
| Precise three-anchor forward projection diff, preserved above | 2405 /46, five unified hunks | b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2 |
| Original36ac25-case logic | 26197 /476 | 639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054 |
| Original36ac complete payload | 132725 /723 | 46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866 |
| New complete readable logic below | 39214 /696 | 6ae4fd802a5515aade80b0b46cffe38030d9b4a46db427b32ed47fb9e20e583c |
| New complete serialized payload below | 151979 /1001 | a5d564be3b8a27abaead1b85b7f87cf2e7a0fe755144e48429b4a505118f8697 |

The original controller is209948bytes/2124lines/SHA256
e3bde0116783eccd01bc6c9c3183c06ade99af21d95800fa47327de9c1ca2207.
That is an archive size, not a Git revision. This report does not duplicate it.
An exact immutable pin/body for the separately corrected EOF controller is
not supplied in this reservation. The available36ac archive is the historical
controller. Integration prerequisite: root must supply the full corrected-EOF
source SHA, report path/fence and canonical controller byte/hash identity before
an exact controller patch can be reviewed. No corrected EOF implementation,
workspace change or controller hunk is guessed here.

The new source binding replaces the old c5 baseline with the entire9f32502-byte
baseline and uses the exact existing Sol3/48288 three-anchor projection delta:
the page assignment, projector plus predicate replacement, and successful
public_decision output property. All three unique inverse spans and the entire
five-hunk2405-byte diff reconstruct the complete9f source. The generic inverse
selector now accounts for zero-length unified-diff ranges; it neither skips a
diff line nor synthesizes source. Collector source/data remain exact. The payload
contains complete canonical base64 candidate/baseline/diff/collector DATA, not
dependencies on mutable worktree scripts.

The complete original25 SwitchCase source ranges are byte-identical, in order,
with the same assertions and outcomes. Their fixed groups remain phase_binding5,
secret_lifetime3, helper_handoff5, partial_framing7 and cleanup_latch5. In particular,
the password lifetime spans username/click/snapshot/password cells; first failure,
helper-zero declared read-only handoff, partial drain/cap/deadline, original start,
single-use refs, repeated clear-before-await, unknown join and independent release
checks are not relaxed. The legacy snapshot factory now declares typed same-result
role/name/ref/actions, and the snapshot stub can return a fixed custom result object.
The legacy25 case bodies remain unchanged; their namespace/input guards do not
attest real browser field selection, authorization or credentials.

Only11 projection scenarios are added. They use19 full-cell invocations; together
with the original25's32 planned invocations, complete success would require51.
The existing75-cell,2500-stub-call and512-trace caps are not raised. No case is
declared passed. The fixed count comes from these meaningful paths, not a
combinatorial product of roles, labels, refs or targets:

| New declared case | Planned cells | Meaningful full-cell assertion |
| --- | ---: | --- |
| projection_exact_pairs_and_private_field_omission | 1 | All12 exact pairs appear in canonical order; only the five allowed typed/click associations survive. Raw values, subject/state/URL/header/cookie/error fields and extra advertised actions never cross output/store/metadata/trace sinks. |
| projection_otp_observed_without_action_or_value | 1 | Exact OTP label is observable, but its type/click advertisements yield no actionable ref and no value/empty-state evidence. |
| projection_required_consent_allow_roundtrip | 2 | Required heading and Allow association reach successful output; one explicit bound Allow click clears the ref before stub entry and the next snapshot supplies fresh Username state. |
| projection_optional_consent_continue_roundtrip | 2 | Optional heading and Continue association follow the same explicit roundtrip without inventing consent reachability. |
| projection_role_and_label_rejections | 2 | Wrong-role Allow and noncanonical/private-bearing label each refuse; no safe association is inferred. |
| projection_incomplete_and_error_rejections | 4 | Incomplete, fixed RP error, non-ok status and tool-error snapshots preserve their exact refusals and expose no public_decision in failed output. |
| projection_missing_or_wrong_advertised_action | 1 | Missing Password action, wrong Username/Sign in action and valid Allow with an extra private action show that only the required advertised action is admitted/copied. |
| projection_malformed_refs | 1 | Wrong namespace, negative index, private string and numeric ref are excluded while the fixed public label remains an observation. |
| projection_disjoint_rows_do_not_infer_association | 1 | A content Allow label cannot label an opaque or unrelated typed ref; prior stored associations cannot be joined across snapshots. |
| projection_duplicate_actions_preserve_multiplicity | 1 | Identical valid rows and another valid ref preserve multiplicity/order while observed labels deduplicate; no click/type occurs or automatic choice is made. |
| projection_latest_success_replaces_previous_snapshot | 3 | Two successful outputs exactly match the latest owned public projection; an older ref then refuses with zero input operations. |

The typed shapes are finite synthetic semantic_v2 consumer inputs with explicit
role/name/ref/actions on the SAME returned result. The oracle never associates
content and ref arrays by index or borrows an earlier label/ref. Test-fixture ref
strings are synthetic namespace examples, not receipts or current provider state.
This memory design does not attest that a future nonblank native Driver result
will contain the assumed typed metadata; missing metadata stays absent.
The12 oracle pairs exactly match the pinned candidate table, including source-backed
required/optional consent and OTP wording. The table is data, not a copied projector.

Every new successful path checks the complete emitted public_decision object AND
the owned stored value. Closed shape permits only observed role/name rows and
refs role/name/action/validated-ref rows. Four distinct public synthetic private
sentinels cover password routing, partial/controller data, exceptions and projection
values. All public outputs, owned/observation stores, persisted metadata and traces
are scanned; real secrets/files are never used. The PW slot is intentionally
private until consumption, as in the old model. Raw synthetic snapshots stay only
in the fixture queue/return; none is directly printed.

A preserved source limit is tested truthfully: tool isError can refuse in checked
before page runs, so an earlier safe internal projection can remain in the owned
store. The tool-error case therefore requires the failed output to withhold
public_decision and the original first failure to survive; it does not assert an
unimplemented clear. Incomplete/error/non-ok snapshots processed by page must
replace that internal projection with empty arrays. No source correction is
smuggled into the test model.

The complete cell is wrapped unchanged in an async VM script only in the FUTURE.
Each invocation receives a fresh context and the same serializable synthetic store;
code generation from strings/wasm is disabled. The five exact Python-looking
collector strings are decoded as DATA, matched and stubbed, never sent to Python
or a shell. Fixed synthetic IDs11–25, session/target/tab and lab are not real
ownership proof. Only a future separately reserved Node child belongs to the
actual outer controller. No stub starts or stops a real browser, helper, listener,
HTTP request, native product or credential transfer.

The old persistent30s Node/performance and35s outer budget design remains the
intended envelope. Per-cell timers use the remaining persistent budget; no case
resets it. First unexpected host assertion is remembered even if the candidate
catches it, then stops later cells/cases after the current finite cell/essential
synthetic cleanup. Packet completion is recorded only after meaningful assertions
return; failure code selection never reads a raw exception message/stack.
EOF/actual-exit/group cleanup/capped transport/retention-before-grading/final-clock
controls must come from the separately pinned corrected controller, not an assumed
copy of the historical one. Capacity, loaded Node dependencies and runtime peaks
are UNMEASURED here; this report grants no slot or execution.

The normal child packet remains the same closed15-key schema
riauth.d01-continuation-memory/v1: schema, source, planned, attempted, completed,
completed_groups, cells, stub_counts, assertions_completed,
privacy_checks_completed, first_failure, unreached, elapsed_ms,
full_candidate_only and actual_tools_or_product. planned is exactly the36-row
BINDING.case_plan. Attempted/completed are ordered prefixes; unreached is the
remaining suffix; group totals are derived only from actually completed names.
Counters are true integers, elapsed is finite, and full_candidate_only=true /
actual_tools_or_product=false remain literal. first_failure is null or the closed
case/check pair from the36 names and96-code catalog. The old minimal privacy-failure
fallback packet remains a FAILED packet that cannot satisfy normal closed-schema
grading; raw bounded output must still be retained before refusal.

Exact new seven-field source object for the future controller's EXPECTED_SOURCE:

```json
{
  "candidate_sha256": "505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f",
  "candidate_bytes": 33701,
  "baseline_sha256": "7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8",
  "baseline_bytes": 32502,
  "diff_sha256": "b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2",
  "logic_sha256": "6ae4fd802a5515aade80b0b46cffe38030d9b4a46db427b32ed47fb9e20e583c",
  "full_inverse": true
}
```


The future controller substitution is confined to the exact payload bytes/hash/
base64 DATA and corresponding PLAN, CHECK_NAMES and EXPECTED_SOURCE data.
PAYLOAD_SHA must be
a5d564be3b8a27abaead1b85b7f87cf2e7a0fe755144e48429b4a505118f8697 and
PAYLOAD_BYTES must be151979. Its canonical base64 DATA is202640 ASCII characters,
derived without an extra source LF. PLAN/CHECK_NAMES are the36 rows/96 codes
printed in the full payload's BINDING declaration; EXPECTED_SOURCE is the exact
seven-field object above. The complete data-only assembly source follows.
It returns exact replacement data, not a guessed controller or EOF hunk.
No counterpart helper or new executable source path is created.

After root supplies the immutable corrected-EOF controller body, the next exact
source proposal must identify those data assignments and compare every remaining
controller AST/source span to that corrected pin. It must preserve EOF flags,
actual exit and full capped stdout/stderr retention before grading, owned WNOWAIT
leader/group cleanup,30/35s clocks, caps/private output, first-failure and final
post-persistence clock. A pass must require all36 completed cases, correct source
identity, no child/harness/cleanup failure and a complete retained packet. This
report supplies no third controller correction, Node/toolchain replacement,
threshold change, runtime approval or source patch against unavailable bytes.

Static proof and recorded errors for this phase:

- Acorn parsed original/new logic, complete payload, full9f baseline and full33701
  candidate. The future async wrapper's complete statement-array AST equals
  the unchanged candidate Program statement array; no wrapper was evaluated or VM constructed.
- Whole three-anchor byte inverse and all five unified-diff hunks reconstruct9f.
  Structural inverse removes only the new projector, page assignment and
  successful-output property and restores the original predicate. Whole AST
  equality holds. Baseline/inverse normalized AST SHA256:
  786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74;
  candidate AST SHA256:
  cce3210177ab2e377a58153c62a2c50f047b031f6e979e0ac17d22b4a46235ce.
  Projector source SHA256:
  414fa7865ac064f9c907af5badf4294ebaa43df319ad56facccd6033b7047feb.
- Exact25 legacy SwitchCase source ranges remain in the first25 positions.
  The36 declared switch names exactly equal BINDING.case_plan. The12 oracle
  pairs equal the source table; this is a static data check, not12 observations.
  Five collector declarations remain byte identical; their decoded source is
  still stub DATA. Payload reconstruction and logic/hash checks passed.
- The initial Git lookup of209948 exited128 because a byte count was mistakenly
  treated as a revision. The36ac report then supplied the actual controller
  archive identity. One source-assembler assertion exited1 after stripping the
  original logic's leading blank LF; retaining the exact26197-byte suffix
  corrected the extractor. Two orchestration sources were rejected by JavaScript
  parsing before Python started because Markdown backticks were quoted incorrectly;
  using a data fence delimiter corrected those source assemblers. These are
  static tooling errors, not harness/candidate/runtime failures. No candidate
  or old harness executed during them or the corrections.
- The complete data-only assembly function below is Python AST parsed, not
  imported/called. Acorn source/data proofs used only installed static parser
  tools. No projected test, VM, future constructor, collector, helper, controller,
  native library/product, Driver/browser, secret file, CLI/HTTP/network/Cargo,
  provider query/download, process signal or runtime slot ran or changed.
- The old25 harness remains UNRUN. All historic real failures, UNKNOWN cause/
  sender/cleanup origin and primary D01/D05 gates remain unchanged. Root full
  review and independent changed-design review precede any ONE finite memory
  release; a real Driver-only journey remains HELD. Other workers/reports/
  source/product/config/security/receipt/headers/PAM protections are untouched.
  Root alone integrates, reserves runtime and owns statuses/publication.

### Complete data-only payload/controller substitution assembly

```python
# Source/data assembly only; this function is NOT called in this phase.
import base64,hashlib,json,re

def projection_payload_data(report_text):
    fence=re.escape(chr(96)*3)
    blocks=re.findall("^"+fence+r"javascript\n(.*?)^"+fence+r"\s*$",report_text,re.M|re.S)
    sha=lambda value:hashlib.sha256(value.encode("utf-8")).hexdigest()
    logic=next(value for value in blocks if
        sha(value)=="6ae4fd802a5515aade80b0b46cffe38030d9b4a46db427b32ed47fb9e20e583c")
    payload=next(value for value in blocks if
        sha(value)=="a5d564be3b8a27abaead1b85b7f87cf2e7a0fe755144e48429b4a505118f8697")
    binding=json.loads(re.search(r"^const BINDING = (\{.*?^\});\n",payload,re.M|re.S)[1])
    assembled="\n".join(payload.splitlines()[:3])+"\nconst BINDING = "+json.dumps(
        binding,indent=2,ensure_ascii=True)+";\n"+logic
    if assembled!=payload or len(payload.encode())!=151979 or not payload.endswith("\n"):
        raise ValueError("payload_identity")
    source={
        "candidate_sha256":binding["candidate_sha256"],"candidate_bytes":33701,
        "baseline_sha256":binding["baseline_sha256"],"baseline_bytes":32502,
        "diff_sha256":binding["diff_sha256"],"logic_sha256":binding["logic_sha256"],
        "full_inverse":True
    }
    compact=lambda value:json.dumps(value,ensure_ascii=True,separators=(",",":"))
    # Exact replacement DATA only, not a patch against an unknown controller.
    return {
        "PAYLOAD_SHA":sha(payload),"PAYLOAD_BYTES":151979,
        "PAYLOAD_B64":base64.b64encode(payload.encode()).decode("ascii"),
        "PLAN_JSON":compact(binding["case_plan"]),
        "CHECK_NAMES_JSON":compact(binding["check_names"]),
        "EXPECTED_SOURCE_JSON":compact(source)
    }
```

### Complete readable9f baseline — immutable source DATA, UNEXECUTED

```javascript
// Prospective ONE functions.exec cell; NOT executed in this design phase.
const CLOCK="import json,time\nprint(json.dumps({'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}))\n";
const STOP="import json,os,pathlib,stat,sys,time\nc=json.loads(sys.argv[1])\nclock={'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())}\nrecord={'schema':'riauth.d01-first-observation/v1','first_failure':c['first_failure'],'first_observation_wall_ms':c['first_observation_wall_ms'],'first_event_proven':False,'clock':clock}\nevent_state='write_unconfirmed'\ntry:\n    fd=os.open(pathlib.Path(c['event_out']),os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n    with os.fdopen(fd,'w',encoding='ascii') as f:\n        f.write(json.dumps(record,sort_keys=True)+'\\n');f.flush();os.fsync(f.fileno())\n    event_state='written'\nexcept OSError:pass\n# Attempt clock persistence BEFORE marker/state/budget comparisons.\n# A metadata error does not prevent the original essential stop protocol.\nlab=pathlib.Path(c['lab']);marker_state='lab_absent'\ntry:\n    if lab.exists():\n        info=lab.lstat()\n        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():\n            marker_state='ownership_unknown'\n        else:\n            marker_state='stop_requested'\n            for name in (('ui-failure','stop') if c['first_failure'] is not None else ('stop',)):\n                try:\n                    fd=os.open(lab/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\n                    os.close(fd)\n                except FileNotFoundError:marker_state='lab_absent'\n                except FileExistsError:pass\nexcept OSError:marker_state='stop_unconfirmed'\nprint(json.dumps({'clock':clock,'event_state':event_state,'marker_state':marker_state}))\n";
const READBACK="import json,os,pathlib,stat,subprocess,sys,time\nc=json.loads(sys.argv[1])\nchildren=None;helper_pid=c['helper_pid'];outer_observation=None;projection='unavailable'\ndef finite_observation(value):\n    if type(value) is not dict or set(value)!= {'observed','diagnostic'} or type(value['observed']) is not bool:\n        return None\n    diagnostic=value['diagnostic']\n    if diagnostic is None:\n        return {'observed':value['observed'],'diagnostic':None}\n    if not value['observed'] or type(diagnostic) is not dict or set(diagnostic)!= {'site','exception_class','own_function','own_line'}:\n        return None\n    sites=('handler','server','main')\n    kinds=('AttributeError','TypeError','ValueError','KeyError','OSError','BrokenPipeError','ConnectionResetError','TimeoutError','other')\n    functions=('HeaderReader.readline','DemoServer.process_request','Demo.begin','Demo.callback','Demo.invoke','Handler.handle_one_request','Handler.send_error','Handler.get','Handler.reply','main')\n    site,kind,function,line=(diagnostic[k] for k in ('site','exception_class','own_function','own_line'))\n    if type(site) is not str or site not in sites or type(kind) is not str or kind not in kinds:\n        return None\n    if not ((function is None and line is None) or (type(function) is str and function in functions and type(line) is int and 1<=line<=1024)):\n        return None\n    return {'observed':True,'diagnostic':{'site':site,'exception_class':kind,'own_function':function,'own_line':line}}\nouter=pathlib.Path(c['outer_out'])\nif outer.is_file():\n    fd=os.open(outer,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)\n    try:\n        info=os.fstat(fd)\n        if not (stat.S_ISREG(info.st_mode) and info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o600 and 0<info.st_size<=262144):\n            raise ValueError('fixed_outer_evidence_invalid')\n        with os.fdopen(fd,'rb',closefd=False) as source:raw_bytes=source.read(262145)\n        if not 0<len(raw_bytes)<=262144:raise ValueError('fixed_outer_evidence_invalid')\n        data=json.loads(raw_bytes)\n    finally:os.close(fd)\n    outer_observation=finite_observation(data.get('unexpected_failure_observation'))\n    projection=data.get('unexpected_observation_projection')\n    if projection not in ('unavailable','valid','invalid'):projection='invalid'\n    if projection=='valid' and outer_observation is None:projection='invalid'\n    allowed={'whoami','discovery','confidential_client_create','operator_login','server','maintenance_init'}\n    started=data.get('helper_invocations')==1 and type(data.get('helper_pid')) is int and data['helper_pid']>0\n    if started:allowed.add('helper')\n    raw=data.get('owned_child_exits')\n    if isinstance(raw,list) and len(raw)==len(allowed) and all(isinstance(v,dict) and v.get('name') in allowed and type(v.get('pid')) is int and v['pid']>0 and type(v.get('exit')) is int for v in raw) and {v['name'] for v in raw}==allowed and len({v['pid'] for v in raw})==len(allowed) and next(v['pid'] for v in raw if v['name']=='server')==c['server_pid'] and ((started and next(v['pid'] for v in raw if v['name']=='helper')==data['helper_pid'] and (helper_pid is None or helper_pid==data['helper_pid'])) or (not started and helper_pid is None and data.get('helper_invocations') is None and data.get('helper_pid') is None)):\n        children=[{k:v[k] for k in ('name','pid','exit')} for v in raw]\n        helper_pid=data['helper_pid'] if started else None\npids=list(c['owned_pids'])\nif helper_pid is not None and helper_pid not in pids:pids.append(helper_pid)\np=subprocess.run(['/bin/ps','-p',','.join(str(v) for v in pids),'-o','pid='],capture_output=True,timeout=3)\nps_known=p.returncode in (0,1) and all(v.isdigit() for v in p.stdout.split())\npresent=set(int(v) for v in p.stdout.split()) if ps_known else set()\nports={}\nfor port in (9000,3000):\n    p=subprocess.run(['/usr/sbin/lsof','-nP','-t','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,timeout=3)\n    ports[str(port)]=not bool(p.stdout.strip()) if p.returncode in (0,1) else None\nprint(json.dumps({'clock':{'monotonic_ns':str(time.monotonic_ns()),'wall_epoch_ns':str(time.time_ns())},'owned_pids':{str(v):(v not in present if ps_known else None) for v in pids},'ports':ports,'lab_absent':not pathlib.Path(c['lab']).exists(),'owned_child_exits':children,'helper_pid':helper_pid,'unexpected_failure_observation':outer_observation,'unexpected_observation_projection':projection}))\n";
const MARKER="import os,pathlib,stat,sys\nlab=pathlib.Path(sys.argv[1]);guard=int(sys.argv[2]);server=int(sys.argv[3])\nif guard<=0 or server<=0 or guard==server:raise ValueError('owned_context_invalid')\ninfo=lab.lstat()\nif not (stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode)==0o700 and info.st_uid==os.getuid()):raise ValueError('owned_context_invalid')\nos.kill(guard,0);os.kill(server,0)\nif (lab/'stop').exists() or (lab/'ui-failure').exists():raise ValueError('owned_context_invalid')\nfd=os.open(lab/'browser-prepared',os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nos.close(fd)\nprint('{\"prepared_marker_written\":true}')\n";
const PERSIST="import json,os,pathlib,sys\npath=pathlib.Path(sys.argv[1])\nrecord=json.loads(sys.argv[2])\n# Convert decimal strings directly to Python integers, avoiding JS Number rounding.\ndef clocks(value):\n    if isinstance(value,dict):\n        for k,v in list(value.items()):\n            if k in ('monotonic_ns','wall_epoch_ns') and isinstance(v,str):\n                assert v.isdecimal() and len(v)<=19 and 0<=int(v)<2**63\n                value[k]=int(v)\n            else:clocks(v)\n    elif isinstance(value,list):\n        for v in value:clocks(v)\nclocks(record)\nfd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)\nwith os.fdopen(fd,'w',encoding='ascii') as f:\n    f.write(json.dumps(record,sort_keys=True,indent=2)+'\\n');f.flush();os.fsync(f.fileno())\nprint(json.dumps({'written_exclusive':True}))\n";
const own=load("d01_immediate_owned_handles");
const OBSKEY="d01_immediate_observation_record";
if (!own || own.runtime_release!==true || own.driver_owned!==true ||
    own.exact_bound!==true || own.single_controller!==true ||
    !["prepared_entry","browser_decision"].includes(own.phase) ||
    (own.phase==="prepared_entry"&&(own.prepared_gate!==true||own.helper_pid!==null||
      own.fixture_ready===true||own.listener_pid_proof===true)) ||
    (own.phase==="browser_decision"&&(own.fixture_ready!==true||own.listener_pid_proof!==true)) ||
    own.fresh_paths_preflight!==true ||
    !Number.isSafeInteger(own.start_epoch_ms) || typeof own.workspace!=="string" ||
    new Set([own.guard_pid,own.server_pid,own.browser_pid,
       ...(own.helper_pid===null?[]:[own.helper_pid])]).size!==(own.helper_pid===null?3:4) ||
    ![own.guard_pid,own.server_pid,own.browser_pid,own.exec_session,
       ...(own.helper_pid===null?[]:[own.helper_pid])]
       .every(v=>Number.isSafeInteger(v)&&v>0) ||
    ![own.session,own.target_id,own.tab_id,own.lab,own.outer_out,
       own.event_out,own.cleanup_out].every(v=>typeof v==="string"&&v.length>0)) {
  text({proposal_refused:"fresh_owned_context_required"});exit();
}
const prior=load(OBSKEY);
if(own.closed===true||prior?.cleanup_started===true){
  text({proposal_refused:"fixture_already_stopped",resource_release_proven:false});exit();
}
const record=prior??{
  schema:"riauth.d01-immediate-observation-cleanup/v1",
  first_failure:null,first_observation_wall_ms:null,
  first_event_proven:false,whole_cleanup_within60_proven:false,
  unexpected_failure_observation:null,diagnostic_errors:[],cleanup_started:false,
  first_clock:null,observation_receipts:[],controller_observations:[],local_tool_receipts:[],actions:[],cleanup_errors:[],
  final_absence:null,owned_child_exits:null,controller_exit:null,
  first_observation_to_final_wall_ms:null,latch_to_absence_ms:null
};
const sq=s=>"'"+String(s).replace(/'/g,"'\\''")+"'";
const retain=()=>store(OBSKEY,record);
const cleanupError=label=>{
  if(!record.cleanup_errors.includes(label))record.cleanup_errors.push(label);
  retain();
};
const local=async(source,arg)=>{
  const cmd="python3 -c "+sq(source)+(arg===undefined?"":" "+sq(JSON.stringify(arg)));
  const r=await tools.exec_command({cmd,workdir:own.workspace,
    yield_time_ms:10000,max_output_tokens:2000});
  record.local_tool_receipts.push({
    label:source===CLOCK?"clock":source===STOP?"stop":source===READBACK?"readback":"unknown",
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    session_id:typeof r.session_id==="number"?r.session_id:null
  });retain(); // Numeric collector receipt BEFORE comparisons.
  if(r.session_id!==undefined) {
    cleanupError("owned_local_command_unjoined");throw new Error("local_pending");
  }
  if(r.exit_code!==0)throw new Error("local_failed");
  return JSON.parse(r.output);
};
function finiteObservation(value) {
  const exact=(v,keys)=>v!==null&&typeof v==="object"&&!Array.isArray(v)&&
    Object.keys(v).length===keys.length&&keys.every(k=>Object.hasOwn(v,k));
  if(!exact(value,["observed","diagnostic"])||typeof value.observed!=="boolean")return null;
  const d=value.diagnostic;
  if(d===null)return {observed:value.observed,diagnostic:null};
  if(!value.observed||!exact(d,["site","exception_class","own_function","own_line"])||
     !["handler","server","main"].includes(d.site)||
     !["AttributeError","TypeError","ValueError","KeyError","OSError","BrokenPipeError",
       "ConnectionResetError","TimeoutError","other"].includes(d.exception_class))return null;
  const functions=["HeaderReader.readline","DemoServer.process_request","Demo.begin","Demo.callback",
    "Demo.invoke","Handler.handle_one_request","Handler.send_error","Handler.get","Handler.reply","main"];
  if(!((d.own_function===null&&d.own_line===null)||
       (functions.includes(d.own_function)&&Number.isSafeInteger(d.own_line)&&
        d.own_line>=1&&d.own_line<=1024)))return null;
  return {observed:true,diagnostic:{site:d.site,exception_class:d.exception_class,
    own_function:d.own_function,own_line:d.own_line}};
}
function diagnostic(value,projection) {
  const projected=finiteObservation(value);
  if(projection==="invalid"||!["valid","unavailable"].includes(projection)||
     (projection==="valid"&&projected===null)) {
    if(!record.diagnostic_errors.includes("observer_projection_invalid"))
      record.diagnostic_errors.push("observer_projection_invalid");
  }
  if(projected!==null&&record.unexpected_failure_observation===null)
    record.unexpected_failure_observation=projected;
  retain(); // No raw diagnostic fallback and no first-failure or outcome mutation.
}
const ns=c=>BigInt(c.monotonic_ns);
const getClock=()=>local(CLOCK);
let controllerJoined=false;
let controllerPollAvailable=true;
let controllerBuffer="";
let cleanupStarted=false;
let lastSnapshotFlags=null;
function latch(label,receivedWall) {
  if(record.first_failure===null) {
    record.first_failure=label;
    record.first_observation_wall_ms=receivedWall;
    retain(); // Synchronous first-failure/wall latch BEFORE any new await/output.
  }
}
function observeController(r,receivedWall) {
  const observation={received_wall_ms:receivedWall,
    exit:typeof r.exit_code==="number"?r.exit_code:null,
    helper_completed:false,helper_exit:null,fixture_finished:false};
  // Complete numeric result projection is retained BEFORE comparisons.
  record.controller_observations.push(observation);retain();
  if(typeof r.exit_code==="number") {
    controllerJoined=true;record.controller_exit=r.exit_code;retain();
  }
  controllerBuffer+=typeof r.output==="string"?r.output:"";
  if(controllerBuffer.length>16384) {
    latch("controller_observation_invalid",receivedWall);controllerBuffer="";return;
  }
  let cut;
  while((cut=controllerBuffer.indexOf("\n"))>=0) {
    const line=controllerBuffer.slice(0,cut);controllerBuffer=controllerBuffer.slice(cut+1);
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch{
      latch("controller_observation_invalid",receivedWall);continue;
    }
    if(Object.hasOwn(event,"unexpected_failure_observation"))
      diagnostic(event.unexpected_failure_observation,event.unexpected_observation_projection);
    if(event.fixture_ready===true) {
      if(event.guard_pid!==own.guard_pid||event.server_pid!==own.server_pid||
         event.lab!==own.lab||!Number.isSafeInteger(event.helper_pid)||event.helper_pid<=0||
         [own.guard_pid,own.server_pid,own.browser_pid].includes(event.helper_pid)||
         (own.helper_pid!==null&&own.helper_pid!==event.helper_pid))
        latch("fixture_ready_unconfirmed",receivedWall);
      else {
        own.helper_pid=event.helper_pid;own.fixture_ready=true;own.listener_pid_proof=true;
        store("d01_immediate_owned_handles",own);
      }
    }
    if(event.helper_completed===true) {
      observation.helper_completed=true;
      observation.helper_exit=typeof event.exit==="number"?event.exit:null;
      retain();
      if(event.exit!==0)latch("helper_failed",receivedWall);
      else if(!cleanupStarted&&record.protected_after_page!==true&&
              !(own.phase==="browser_decision"&&
                own.next_decision?.kind==="protected_after"))
        latch("helper_completed_before_app_checkpoint",receivedWall);
    }
    if(event.fixture_finished===true) {
      observation.fixture_finished=true;retain();
      if(!cleanupStarted)latch("controller_completed_before_app_checkpoint",receivedWall);
    }
  }
  if(controllerJoined&&!cleanupStarted)
    latch("controller_completed_before_app_checkpoint",receivedWall);
}
async function pollController() {
  if(controllerJoined||!controllerPollAvailable)return;
  try {
    const r=await tools.write_stdin({session_id:own.exec_session,
      chars:"",yield_time_ms:5000,max_output_tokens:2000});
    const receivedWall=Date.now();
    observeController(r,receivedWall);
  } catch {
    controllerPollAvailable=false;
    latch("controller_observation_unavailable",Date.now());
    if(cleanupStarted)cleanupError("controller_join_unconfirmed");
  }
}
async function timedDriver(label,allocation,operation,project) {
  let start=null,end=null,result=null,state="unknown";
  try{start=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label,start_clock:start,end_clock:null,allowed_ms:allocation,
    result_state:"unknown",elapsed_ms:null,over_budget:null};
  record.actions.push(action);retain(); // Start retained BEFORE entering Driver call.
  try {
    result=await operation();
    state=result?.isError===true?"refused":"returned";
  } catch {state="exception";}
  try{end=await getClock();}catch{cleanupError("clock_unavailable");}
  action.end_clock=end;action.result_state=state;
  retain(); // End/result retained BEFORE budget comparison.
  if(start&&end) {
    const d=ns(end)-ns(start);
    if(d>=0n) {
      action.elapsed_ms=Number(d/1000000n);
      action.over_budget=action.elapsed_ms>allocation;
    } else cleanupError("clock_invalid");
  }
  if(action.over_budget)cleanupError("driver_operation_over_budget");
  if(state!=="returned")cleanupError("driver_operation_unconfirmed");
  if(project)project(result,state);
  retain();
}
async function cleanup() {
  store("d01_fresh_password_input",null); // Clear before any cleanup await.
  if(cleanupStarted)return;
  cleanupStarted=true;record.cleanup_started=true;retain();
  try {
    const r=await local(STOP,{
      lab:own.lab,event_out:own.event_out,
      first_failure:record.first_failure,
      first_observation_wall_ms:record.first_observation_wall_ms
    });
    record.first_clock=r.clock;record.stop_state=r.marker_state;retain();
    if(r.event_state!=="written")cleanupError("observation_metadata_write_unconfirmed");
    if(r.marker_state==="stop_unconfirmed")cleanupError("stop_unconfirmed");
    if(r.marker_state==="ownership_unknown")cleanupError("ownership_unknown");
  } catch {cleanupError("stop_or_clock_record_unavailable");}
  // No outstanding Driver call exists here: all page calls above were awaited.
  // Original stop protocol already causes the controller's owned-child finally.
  await timedDriver("kill_app",30000,
    ()=>tools.mcp__cua_driver__kill_app({pid:own.browser_pid}));
  await timedDriver("end_session",15000,
    ()=>tools.mcp__cua_driver__end_session({session:own.session}),(r,state)=>{
      record.session_ended=state==="returned"&&
        r?.structuredContent?.active===false&&r.structuredContent.session===own.session;
      retain();
    });
  await timedDriver("list_windows",5000,
    ()=>tools.mcp__cua_driver__list_windows({pid:own.browser_pid}),(r,state)=>{
      const windows=r?.structuredContent?.windows;
      record.window_count=state==="returned"&&Array.isArray(windows)?windows.length:null;
      retain();
    });
  let readStart=null;
  try{readStart=await getClock();}catch{cleanupError("clock_unavailable");}
  const action={label:"owned_join_readback",start_clock:readStart,end_clock:null,
    allowed_ms:null,elapsed_ms:null,over_budget:null,result_state:"unknown"};
  record.actions.push(action);retain();
  if(readStart&&record.first_clock) {
    action.allowed_ms=Math.max(0,60000-Number((ns(readStart)-ns(record.first_clock))/1000000n));
    retain();
  }
  // Continue essential join after60s if late; never claim that deadline enforced.
  // Do not poll another exec session or send a manual process signal.
  while(!controllerJoined&&controllerPollAvailable&&Date.now()<own.start_epoch_ms+900000) {
    await pollController();
    if(record.first_clock) {
      try {
        const c=await getClock();record.last_join_clock=c;retain();
        if(ns(c)-ns(record.first_clock)>60000000000n)
          cleanupError("cleanup_budget_exceeded");
      } catch{cleanupError("clock_unavailable");}
    }
  }
  if(!controllerJoined)cleanupError("child_reap_incomplete");
  try {
    const r=await local(READBACK,{
      owned_pids:[own.guard_pid,own.server_pid,own.browser_pid,
        ...(own.helper_pid===null?[]:[own.helper_pid])],
      lab:own.lab,outer_out:own.outer_out,helper_pid:own.helper_pid,server_pid:own.server_pid
    });
    action.end_clock=r.clock;action.result_state="returned";
    record.owned_child_exits=r.owned_child_exits;
    diagnostic(r.unexpected_failure_observation,r.unexpected_observation_projection);
    if(Number.isSafeInteger(r.helper_pid)&&r.helper_pid>0)own.helper_pid=r.helper_pid;
    record.final_absence={owned_pids:r.owned_pids,ports:r.ports,
      lab:r.lab_absent,window_count:record.window_count??null,
      session_ended:record.session_ended===true};
    record.first_observation_to_final_wall_ms=record.first_observation_wall_ms===null?null:
      Date.now()-record.first_observation_wall_ms;
    retain(); // Full fixed readback/exits/clock BEFORE comparisons.
    if(readStart) {
      action.elapsed_ms=Number((ns(r.clock)-ns(readStart))/1000000n);
      action.over_budget=action.allowed_ms===null?null:action.elapsed_ms>action.allowed_ms;
    }
    if(record.first_clock)
      record.latch_to_absence_ms=Number((ns(r.clock)-ns(record.first_clock))/1000000n);
    if(action.over_budget)cleanupError("cleanup_budget_exceeded");
    const a=record.final_absence;
    if(!controllerJoined||!Array.isArray(record.owned_child_exits)||
       record.owned_child_exits.length!==(own.helper_pid===null?6:7))
      cleanupError("child_reap_incomplete");
    if(!Object.values(a.owned_pids).every(v=>v===true)||
       !Object.values(a.ports).every(v=>v===true)||a.lab!==true||
       a.window_count!==0||a.session_ended!==true)
      cleanupError("owned_resource_absence_unproven");
  } catch {action.result_state="exception";cleanupError("owned_readback_unavailable");}
  cleanupError("first_event_unproven");
  record.whole_cleanup_within60_proven=false;retain();
  // Exclusive new file only; existing outer/helper/provider evidence untouched.
  try {
    const cmd="python3 -c "+sq(PERSIST)+" "+sq(own.cleanup_out)+" "+sq(JSON.stringify(record));
    const r=await tools.exec_command({cmd,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500});
    record.local_tool_receipts.push({label:"persist",
      exit:typeof r.exit_code==="number"?r.exit_code:null,
      session_id:typeof r.session_id==="number"?r.session_id:null});retain();
    if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
    if(r.session_id!==undefined||r.exit_code!==0)
      cleanupError("cleanup_metadata_write_unconfirmed");
  } catch {cleanupError("cleanup_metadata_write_unconfirmed");}
  controllerBuffer="";own.closed=true;store("d01_immediate_owned_handles",own);
}
async function checked(label,operation,predicate) {
  if(record.first_failure!==null)return null;
  if(Date.now()>=own.start_epoch_ms+840000) {
    latch("browser_active_deadline",Date.now());await cleanup();return null;
  }
  let result,receivedWall;
  try {
    result=await operation();receivedWall=Date.now();
    record.observation_receipts.push({kind:label,received_wall_ms:receivedWall});retain();
    if(result?.isError===true)latch("browser_tool_refused",receivedWall);
    else if(!predicate(result))latch("browser_decision_unconfirmed",receivedWall);
  } catch {latch("browser_tool_or_decision_exception",Date.now());}
  if(record.first_failure!==null)await cleanup();
  return record.first_failure===null?result:null;
}
const snapshotArgs={session:own.session,target_id:own.target_id,tab_id:own.tab_id,
  snapshot_format:"semantic_v2",include_screenshot:false};
function page(result,kind) {
  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];
  const flags={
    status_ok:result?.isError!==true&&s?.status==="ok",
    complete:s?.snapshot?.complete===true,
    heading_match:nodes.some(n=>n.role==="heading"&&n.name===
      (kind==="protected_after"?"Protected application access":"Local demo")),
    error_match:nodes.some(n=>n.name==="Local demo could not complete this request."),
    required_text:nodes.some(n=>n.name===(kind==="protected_before"?"Sign in required.":
      kind==="protected_after"?"Signed in. Protected application access is available.":
      "Use Sign in to open this local application.")),
    interactive_ref_present:Array.isArray(s?.refs)&&
      s.refs.some(n=>Array.isArray(n.actions)&&n.actions.includes("click"))
  };
  record.last_page_flags={kind,...flags};retain();
  // Only provider refs and fixed public-label matches survive this raw snapshot.
  own.fresh_refs=Array.isArray(s?.refs)?s.refs.filter(n=>typeof n.ref==="string"&&
    /^p[0-9]+:[0-9]+$/.test(n.ref)).map(n=>n.ref):[];
  store("d01_immediate_owned_handles",own);
  if(flags.error_match)return false;
  if(!flags.status_ok||!flags.complete)return false;
  if(kind==="generic_checked") {
    if(nodes.some(n=>n.role==="heading"&&n.name==="Protected application access")&&
       nodes.some(n=>n.name==="Signed in. Protected application access is available."))
      record.protected_after_page=true;
    return true;
  }
  const ok=flags.heading_match&&flags.required_text&&
    (kind!=="application"||flags.interactive_ref_present);
  if(ok&&kind==="protected_after")record.protected_after_page=true;
  return ok;
}
async function navigateAndSnapshot(url,kind) {
  if(await checked("browser_navigation",
      ()=>tools.mcp__cua_driver__browser_navigate({session:own.session,
        target_id:own.target_id,tab_id:own.tab_id,url}),
      r=>r?.isError!==true)) {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,kind));
  }
}
async function entry() {
  // The exact Driver-owned blank handles already exist while the180s gate waits.
  // There is no model yield, tool-description load or rebind after this marker.
  const markerCommand="python3 -c "+sq(MARKER)+" "+sq(own.lab)+" "+
    sq(own.guard_pid)+" "+sq(own.server_pid);
  await checked("prepared_marker",
    ()=>tools.exec_command({cmd:markerCommand,workdir:own.workspace,
      yield_time_ms:10000,max_output_tokens:500}),r=>{
      record.local_tool_receipts.push({label:"marker",
        exit:typeof r.exit_code==="number"?r.exit_code:null,
        session_id:typeof r.session_id==="number"?r.session_id:null});retain();
      if(r.session_id!==undefined)cleanupError("owned_local_command_unjoined");
      return r.exit_code===0&&r.session_id===undefined&&
        JSON.parse(r.output).prepared_marker_written===true;
    });
  const readyDeadline=Date.now()+30000;
  while(record.first_failure===null&&own.fixture_ready!==true&&Date.now()<readyDeadline)
    await pollController();
  if(record.first_failure===null&&own.fixture_ready!==true)
    latch("fixture_ready_unconfirmed",Date.now());
  if(record.first_failure!==null){await cleanup();return;}
  await pollController(); // ONE immediate pre-navigation poll, no RP HTTP probe.
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/protected","protected_before");
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  await navigateAndSnapshot("http://localhost:3000/","application");
  if(record.first_failure===null)await pollController(); // ONE post-entry poll.
  if(record.first_failure===null) {
    own.phase="browser_decision";
    store("d01_immediate_owned_handles",own);
  }
  if(record.first_failure!==null)await cleanup();
}
async function continuation() {
  const spec=own.next_decision;
  const allowedKinds=["click","username","password","snapshot","protected_after"];
  if(!spec||!allowedKinds.includes(spec.kind)) {
    latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
  }
  await pollController();
  if(record.first_failure!==null){await cleanup();return;}
  if(spec.kind==="protected_after") {
    // Read the fresh callback-following protected page; do not send another RP request.
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),r=>page(r,"protected_after"));
  } else if(spec.kind==="snapshot") {
    await checked("browser_snapshot",
      ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
      r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  } else {
    if(typeof spec.ref!=="string"||!Array.isArray(own.fresh_refs)||
       !own.fresh_refs.includes(spec.ref)) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    own.fresh_refs=[];store("d01_immediate_owned_handles",own); // Single-use snapshot ref.
    const args={session:own.session,target_id:own.target_id,tab_id:own.tab_id,ref:spec.ref};
    const operation=spec.kind==="click"?
      ()=>tools.mcp__cua_driver__browser_click({...args,input_route:"dom_event"}):
      ()=>tools.mcp__cua_driver__browser_type({...args,replace:true,
        text:spec.kind==="username"?"admin":load("d01_fresh_password_input")});
    if(spec.kind==="password"&&(typeof load("d01_fresh_password_input")!=="string"||
       !/^[A-Za-z0-9_-]{30,128}$/.test(load("d01_fresh_password_input")))) {
      latch("browser_decision_unconfirmed",Date.now());await cleanup();return;
    }
    await checked("browser_input",operation,r=>{
      const s=r?.structuredContent;
      return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
    }); // Dispatch alone never earns an application outcome or journey credit.
    if(spec.kind==="password")store("d01_fresh_password_input",null);
    if(record.first_failure===null)
      await checked("browser_snapshot",
        ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
        r=>page(r,"generic_checked")&&publicPredicate(r,spec));
  }
  if(record.first_failure===null)await pollController();
  if(record.first_failure!==null)await cleanup();
  else if(spec.kind==="protected_after") {
    record.cleanup_requested_wall_ms=Date.now();retain();
    await cleanup();
  }
}
function publicPredicate(result,spec) {
  const nodes=result?.structuredContent?.content_refs;
  // Root must pin one printed public expected label/role before that action.
  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.
  const roles=["heading","button","statictext","textbox"];
  const labels=["Username","Password","Authenticator or recovery code","Sign in",
    "Local demo","Protected application access",
    "Signed in. Protected application access is available."];
  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&
    labels.includes(spec.expected_label)&&
    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);
}
try {
  if(own.phase==="prepared_entry")await entry();else await continuation();
  // Do not carry unvalidated partial controller text across this cell boundary.
  while(controllerBuffer.length>0&&record.first_failure===null) {
    const beforePollWall=Date.now();
    if(beforePollWall>=own.start_epoch_ms+840000) {
      latch("browser_active_deadline",beforePollWall);break;
    }
    if(controllerJoined||!controllerPollAvailable) {
      latch("controller_observation_invalid",beforePollWall);break;
    }
    await pollController(); // Same owned session; observer retains receipt first.
    const afterPollWall=Date.now();
    if(afterPollWall>=own.start_epoch_ms+840000)
      latch("browser_active_deadline",afterPollWall);
  }
  if(record.first_failure!==null)await cleanup();
} catch {
  latch("composed_decision_exception",Date.now());
  await cleanup();
}
if(record.first_failure!==null) {
  text({result:"failed",first_failure:record.first_failure,
    unexpected_failure_observation:record.unexpected_failure_observation,
    diagnostic_errors:record.diagnostic_errors,cleanup_errors:record.cleanup_errors,
    final_absence:record.final_absence,controller_exit:record.controller_exit,
    whole_cleanup_within60_proven:false,
    resource_release_proven:record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v))});
} else {
  text({result:"browser_decision_observed_only",page_flags:record.last_page_flags??null,
    journey_credit:false,cleanup_errors:record.cleanup_errors,
    resource_release_proven:record.cleanup_started===true&&record.final_absence!==null&&
      !record.cleanup_errors.some(v=>[
        "child_reap_incomplete","owned_resource_absence_unproven",
        "owned_readback_unavailable","owned_local_command_unjoined"].includes(v)),
    whole_cleanup_within60_proven:false});
}
```

### Exact harness logic diff — proposed archive changes only

Harness diff: 15155bytes/258lines/SHA256 b22942272d11b9e397f00e8eb799954a2be02f20a2ae2824810b08f888a79117.

```diff
--- 36ac893-readable-logic
+++ DESIGN-public-projection-readable-logic
@@ -6 +6,2 @@
-const PRIVATE=[PASSWORD,PARTIAL,EXCEPTION];
+const PROJECTION="D01_SYNTHETIC_PROJECTION_SENTINEL_PRIVATE";
+const PRIVATE=[PASSWORD,PARTIAL,EXCEPTION,PROJECTION];
@@ -32,2 +33,2 @@
-  ensure(ds[0]==="--- immutable-c5d4d173-cell-d7278780\n"&&
-    ds[1]==="+++ DESIGN-only-F2-F3-F4\n","diff_headers");
+  ensure(ds[0]===BINDING.diff_headers[0]&&
+    ds[1]===BINDING.diff_headers[1],"diff_headers");
@@ -36 +37,2 @@
-    ensure(m!==null,"diff_hunk");hunks++;const start=Number(m[3])-1;
+    ensure(m!==null,"diff_hunk");hunks++;const count=m[4]===undefined?1:Number(m[4]);
+    const start=count===0?Number(m[3]):Number(m[3])-1;
@@ -45 +47 @@
-  ensure(hunks===4,"diff_hunk_count");out.push(...lines.slice(cursor));return out.join("");
+  ensure(hunks===BINDING.diff_hunks,"diff_hunk_count");out.push(...lines.slice(cursor));return out.join("");
@@ -51 +53 @@
-  ensure(Buffer.byteLength(CANDIDATE)===32502&&sha(CANDIDATE)===BINDING.candidate_sha256,
+  ensure(Buffer.byteLength(CANDIDATE)===33701&&sha(CANDIDATE)===BINDING.candidate_sha256,
@@ -53 +55 @@
-  ensure(Buffer.byteLength(BASELINE)===31581&&sha(BASELINE)===BINDING.baseline_sha256,
+  ensure(Buffer.byteLength(BASELINE)===32502&&sha(BASELINE)===BINDING.baseline_sha256,
@@ -55 +57 @@
-  ensure(Buffer.byteLength(diff)===2243&&sha(diff)===BINDING.diff_sha256,"diff_identity");
+  ensure(Buffer.byteLength(diff)===2405&&sha(diff)===BINDING.diff_sha256,"diff_identity");
@@ -86 +88,2 @@
-      {role:"statictext",name:"Use Sign in to open this local application."}]:
+      {role:"statictext",name:"Use Sign in to open this local application."},
+      {role:"button",name:"Sign in"}]:
@@ -89,2 +92,8 @@
-  return {structuredContent:{status:"ok",snapshot:{complete:true},content_refs:nodes,
-    refs:[{ref,actions:["click"]}]}};
+  const base=Number(ref.split(":")[1]),prefix=ref.split(":")[0]+":";
+  const refs=kind==="after"||kind==="before"?[]:
+    kind==="application"?[{role:"button",name:"Sign in",ref,actions:["click"]}]:[
+      {role:"textbox",name:"Username",ref,actions:["type"]},
+      {role:"textbox",name:"Password",ref:prefix+String(base+1000),actions:["type"]},
+      {role:"button",name:"Sign in",ref:prefix+String(base+2000),actions:["click"]}
+    ];
+  return {structuredContent:{status:"ok",snapshot:{complete:true},content_refs:nodes,refs}};
@@ -231,0 +241 @@
+      if(kind!==null&&typeof kind==="object")return clone(kind);
@@ -237,0 +248,2 @@
+      if(s.options.projection_protocol)ensure(load(s,OWN).fresh_refs.length===0&&
+        a.ref===s.options.expected_action_ref,"projection_single_use_before_await");
@@ -303,0 +316,64 @@
+const PUBLIC_PAIRS=[
+  ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
+  ["heading","Local demo wants to use your riAuth account"],
+  ["heading","Continue to Local demo?"],["heading","Protected application access"],
+  ["button","Sign in"],["button","Allow"],["button","Continue"],
+  ["textbox","Username"],["textbox","Password"],
+  ["textbox","Authenticator or recovery code (if enabled)"],
+  ["statictext","Signed in. Protected application access is available."]
+];
+function projectionNode(role,name){
+  return {role,name,value:PROJECTION,url:PROJECTION,subject:PROJECTION,
+    state:PROJECTION,cookie:PROJECTION,headers:{authorization:PROJECTION},error:PROJECTION};
+}
+function projectionRef(role,name,ref,actions){
+  return {...projectionNode(role,name),ref,actions};
+}
+function projectionSnapshot(nodes,refs,changes={}){
+  return {structuredContent:{status:"ok",snapshot:{complete:true},
+    content_refs:nodes,refs,...changes}};
+}
+function projectionDecision(s,kind,role,name,ref){
+  decision(s,kind,ref);const o=load(s,OWN);
+  o.next_decision.expected_role=role;o.next_decision.expected_label=name;s.map.set(OWN,o);
+}
+function keysExactly(value,keys){
+  return value!==null&&typeof value==="object"&&!Array.isArray(value)&&
+    Object.keys(value).sort().join(",")===[...keys].sort().join(",");
+}
+function projectionShape(value){
+  ensure(keysExactly(value,["observed","refs"])&&
+    Array.isArray(value.observed)&&Array.isArray(value.refs),"projection_closed_shape");
+  for(const row of value.observed){
+    ensure(keysExactly(row,["role","name"])&&PUBLIC_PAIRS.some(([role,name])=>
+      row.role===role&&row.name===name),"projection_canonical_pair");
+  }
+  for(const row of value.refs){
+    ensure(keysExactly(row,["role","name","action","ref"])&&PUBLIC_PAIRS.some(([role,name])=>
+      row.role===role&&row.name===name)&&typeof row.ref==="string"&&
+      /^p[0-9]+:[0-9]+$/.test(row.ref),"projection_canonical_ref");
+    ensure((row.role==="button"&&row.action==="click")||
+      (row.role==="textbox"&&["Username","Password"].includes(row.name)&&row.action==="type"),
+      "projection_action_pair");
+  }
+  privacy(value);
+}
+function projected(s,out,expected){
+  finalOutcome(s,null,false);projectionShape(out.public_decision);
+  const owned=load(s,OWN);projectionShape(owned.public_decision);
+  ensure(JSON.stringify(out.public_decision)===JSON.stringify(expected)&&
+    JSON.stringify(owned.public_decision)===JSON.stringify(expected),"projection_exact_output_and_store");
+  return out;
+}
+function projectionFailed(s,out,expected,emptyOwned=true){
+  finalOutcome(s,expected,true);
+  ensure(!Object.hasOwn(out,"public_decision")&&s.inputKinds.length===0,
+    "projection_failure_no_public_decision_or_input");
+  if(emptyOwned)ensure(JSON.stringify(load(s,OWN).public_decision)===
+    '{"observed":[],"refs":[]}',"projection_processed_failure_empty");
+}
+function noProjectionInput(s){
+  ensure(s.inputKinds.length===0&&!s.trace.some(x=>["click","type"].includes(x.kind)),
+    "projection_no_automatic_input");
+}
+
@@ -444,0 +521,143 @@
+    case "projection_exact_pairs_and_private_field_omission":{
+      s=makeState();projectionDecision(s,"snapshot","heading","Local demo");
+      const nodes=PUBLIC_PAIRS.map(([role,name])=>projectionNode(role,name));
+      const refs=[
+        projectionRef("button","Sign in","p70:1",["click",PROJECTION]),
+        projectionRef("button","Allow","p70:2",["click"]),
+        projectionRef("button","Continue","p70:3",["click"]),
+        projectionRef("textbox","Username","p70:4",["type"]),
+        projectionRef("textbox","Password","p70:5",["type"]),
+        projectionRef("textbox","Authenticator or recovery code (if enabled)","p70:6",["type","click"]),
+        projectionRef("heading","Local demo","p70:7",["click"])
+      ];
+      nodes.push(projectionNode("textbox",PROJECTION));
+      s.snapshotQueue=[projectionSnapshot(nodes,refs,{page:{url:PROJECTION},account:{subject:PROJECTION},
+        error:PROJECTION,headers:{authorization:PROJECTION},cookie:PROJECTION,state:PROJECTION})];
+      const out=await cell(s);
+      projected(s,out,{observed:PUBLIC_PAIRS.map(([role,name])=>({role,name})),refs:[
+        {role:"button",name:"Sign in",action:"click",ref:"p70:1"},
+        {role:"button",name:"Allow",action:"click",ref:"p70:2"},
+        {role:"button",name:"Continue",action:"click",ref:"p70:3"},
+        {role:"textbox",name:"Username",action:"type",ref:"p70:4"},
+        {role:"textbox",name:"Password",action:"type",ref:"p70:5"}
+      ]});noProjectionInput(s);break;
+    }
+    case "projection_otp_observed_without_action_or_value":{
+      s=makeState();projectionDecision(s,"snapshot","textbox","Authenticator or recovery code (if enabled)");
+      s.snapshotQueue=[projectionSnapshot([
+        projectionNode("textbox","Authenticator or recovery code (if enabled)")],[
+        projectionRef("textbox","Authenticator or recovery code (if enabled)","p71:1",["type","click"])])];
+      projected(s,await cell(s),{observed:[
+        {role:"textbox",name:"Authenticator or recovery code (if enabled)"}],refs:[]});
+      noProjectionInput(s);break;
+    }
+    case "projection_required_consent_allow_roundtrip":
+    case "projection_optional_consent_continue_roundtrip":{
+      const required=name==="projection_required_consent_allow_roundtrip";
+      const heading=required?"Local demo wants to use your riAuth account":"Continue to Local demo?";
+      const label=required?"Allow":"Continue",ref=required?"p72:1":"p73:1";
+      s=makeState({projection_protocol:true,expected_action_ref:ref});
+      projectionDecision(s,"snapshot","heading",heading);
+      s.snapshotQueue=[projectionSnapshot([
+        projectionNode("heading",heading),projectionNode("button",label)],[
+        projectionRef("button",label,ref,["click"])])];
+      projected(s,await cell(s),{observed:[{role:"heading",name:heading},{role:"button",name:label}],
+        refs:[{role:"button",name:label,action:"click",ref}]});
+      noProjectionInput(s);
+      projectionDecision(s,"click","textbox","Username",ref);
+      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
+        projectionRef("textbox","Username","p74:1",["type"])])];
+      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
+        refs:[{role:"textbox",name:"Username",action:"type",ref:"p74:1"}]});
+      ensure(s.inputKinds.join(",")==="click"&&!load(s,OWN).fresh_refs.includes(ref),
+        "projection_consent_dispatch_and_freshness");break;
+    }
+    case "projection_role_and_label_rejections":{
+      for(const pair of [["heading","Allow"],["button","Allow "+PROJECTION]]){
+        s=makeState();projectionDecision(s,"snapshot","button","Allow");
+        s.snapshotQueue=[projectionSnapshot([projectionNode(...pair)],[
+          projectionRef(...pair,"p75:1",["click"])])];
+        projectionFailed(s,await cell(s),"browser_decision_unconfirmed");
+      }break;
+    }
+    case "projection_incomplete_and_error_rejections":{
+      for(const variant of ["incomplete","fixed_error","bad_status","tool_error"]){
+        s=makeState();projectionDecision(s,"snapshot","textbox","Username");
+        const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
+          refs:[{role:"button",name:"Allow",action:"click",ref:"p76:9"}]};s.map.set(OWN,owned);
+        const raw=projectionSnapshot([projectionNode("textbox","Username")],[
+          projectionRef("textbox","Username","p76:1",["type"])]);
+        if(variant==="incomplete")raw.structuredContent.snapshot.complete=false;
+        if(variant==="fixed_error")raw.structuredContent.content_refs.push(
+          projectionNode("statictext","Local demo could not complete this request."));
+        if(variant==="bad_status")raw.structuredContent.status="refused";
+        if(variant==="tool_error"){raw.isError=true;raw.error=PROJECTION;}
+        s.snapshotQueue=[raw];projectionFailed(s,await cell(s),
+          variant==="tool_error"?"browser_tool_refused":"browser_decision_unconfirmed",
+          variant!=="tool_error");
+      }break;
+    }
+    case "projection_missing_or_wrong_advertised_action":{
+      s=makeState();projectionDecision(s,"snapshot","textbox","Username");
+      const missing=projectionRef("textbox","Password","p77:2",["type"]);delete missing.actions;
+      s.snapshotQueue=[projectionSnapshot([
+        projectionNode("textbox","Username"),projectionNode("textbox","Password"),
+        projectionNode("button","Sign in"),projectionNode("button","Allow")],[
+        projectionRef("textbox","Username","p77:1",["click"]),missing,
+        projectionRef("button","Sign in","p77:3",["type"]),
+        projectionRef("button","Allow","p77:4",["click",PROJECTION])])];
+      projected(s,await cell(s),{observed:[{role:"button",name:"Sign in"},{role:"button",name:"Allow"},
+        {role:"textbox",name:"Username"},{role:"textbox",name:"Password"}],
+        refs:[{role:"button",name:"Allow",action:"click",ref:"p77:4"}]});
+      noProjectionInput(s);break;
+    }
+    case "projection_malformed_refs":{
+      s=makeState();projectionDecision(s,"snapshot","button","Allow");
+      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
+        projectionRef("button","Allow","x1:1",["click"]),
+        projectionRef("button","Allow","p1:-1",["click"]),
+        projectionRef("button","Allow",PROJECTION,["click"]),
+        projectionRef("button","Allow",17,["click"])])];
+      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
+      noProjectionInput(s);break;
+    }
+    case "projection_disjoint_rows_do_not_infer_association":{
+      s=makeState();projectionDecision(s,"snapshot","button","Allow");
+      const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
+        refs:[{role:"button",name:"Allow",action:"click",ref:"p78:9"}]};s.map.set(OWN,owned);
+      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
+        {ref:"p78:1",actions:["click"],value:PROJECTION},
+        projectionRef("button","Unrelated "+PROJECTION,"p78:2",["click"])])];
+      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
+      ensure(!load(s,OWN).fresh_refs.includes("p78:9"),"projection_no_cross_snapshot_join");
+      noProjectionInput(s);break;
+    }
+    case "projection_duplicate_actions_preserve_multiplicity":{
+      s=makeState();projectionDecision(s,"snapshot","button","Allow");
+      const row=projectionRef("button","Allow","p79:1",["click"]);
+      s.snapshotQueue=[projectionSnapshot([
+        projectionNode("button","Allow"),projectionNode("button","Allow")],[
+        row,clone(row),projectionRef("button","Allow","p79:2",["click"])])];
+      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[
+        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
+        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
+        {role:"button",name:"Allow",action:"click",ref:"p79:2"}]});
+      noProjectionInput(s);break;
+    }
+    case "projection_latest_success_replaces_previous_snapshot":{
+      s=makeState();projectionDecision(s,"snapshot","button","Allow");
+      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
+        projectionRef("button","Allow","p80:1",["click"])])];
+      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],
+        refs:[{role:"button",name:"Allow",action:"click",ref:"p80:1"}]});
+      projectionDecision(s,"snapshot","textbox","Username");
+      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
+        projectionRef("textbox","Username","p81:1",["type"])])];
+      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
+        refs:[{role:"textbox",name:"Username",action:"type",ref:"p81:1"}]});
+      ensure(!load(s,OWN).fresh_refs.includes("p80:1"),"projection_latest_ref_only");
+      projectionDecision(s,"click","textbox","Username","p80:1");
+      await cell(s);finalOutcome(s,"browser_decision_unconfirmed",true);
+      ensure(s.inputKinds.length===0,
+        "projection_old_ref_refused");break;
+    }
@@ -458,0 +678 @@
+    ensure(result.cells===51,"projection_complete_cell_count");
```

### Complete readable future harness logic — UNEXECUTED

```javascript

const LIMIT_MS=30000, MAX_CELLS=75, MAX_CALLS=2500, MAX_TRACE=512;
const PASSWORD="D01_SYNTHETIC_PASSWORD_SENTINEL_0123456789ABCDEFG";
const PARTIAL="D01_SYNTHETIC_PARTIAL_SENTINEL_PRIVATE";
const EXCEPTION="D01_SYNTHETIC_EXCEPTION_SENTINEL_PRIVATE";
const PROJECTION="D01_SYNTHETIC_PROJECTION_SENTINEL_PRIVATE";
const PRIVATE=[PASSWORD,PARTIAL,EXCEPTION,PROJECTION];
const OBS="d01_immediate_observation_record", OWN="d01_immediate_owned_handles";
const PW="d01_fresh_password_input", EXIT=Object.freeze({});
const FAULTS=new WeakMap();
const result={schema:"riauth.d01-continuation-memory/v1",
  source:null,planned:BINDING.case_plan,attempted:[],completed:[],
  completed_groups:{},cells:0,stub_counts:{},assertions_completed:0,
  privacy_checks_completed:0,first_failure:null,unreached:[],elapsed_ms:null,
  full_candidate_only:true,actual_tools_or_product:false};
function failure(code){const e=Object.freeze({});FAULTS.set(e,code);return e;}
function ensure(ok,code){if(!ok)throw failure(code);result.assertions_completed++;}
function clockGuard(){ensure(performance.now()<LIMIT_MS,"real_deadline");}
const clone=v=>v===undefined?undefined:JSON.parse(JSON.stringify(v));
const sha=s=>createHash("sha256").update(s).digest("hex");
function decode(encoded){ensure(typeof encoded==="string"&&
  /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(encoded),
  "source_encoding");const b=Buffer.from(encoded,"base64");
  ensure(b.toString("base64")===encoded,"source_encoding");return b.toString("utf8");}
function privacy(value){
  const s=JSON.stringify(value);ensure(typeof s==="string"&&!PRIVATE.some(x=>s.includes(x)),
    "privacy_sink");result.privacy_checks_completed++;
}
function inverseDiff(candidate,diff){
  const lines=candidate.match(/[^\n]*\n/g)||[], ds=diff.match(/[^\n]*\n/g)||[];
  ensure(lines.join("")===candidate&&ds.join("")===diff,"diff_framing");
  let cursor=0,i=2,out=[],hunks=0;
  ensure(ds[0]===BINDING.diff_headers[0]&&
    ds[1]===BINDING.diff_headers[1],"diff_headers");
  while(i<ds.length){
    const m=/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n$/.exec(ds[i++]);
    ensure(m!==null,"diff_hunk");hunks++;const count=m[4]===undefined?1:Number(m[4]);
    const start=count===0?Number(m[3]):Number(m[3])-1;
    ensure(cursor<=start,"diff_position");out.push(...lines.slice(cursor,start));cursor=start;
    while(i<ds.length&&!ds[i].startsWith("@@ ")){
      const kind=ds[i][0],text=ds[i++].slice(1);
      ensure([" ","+","-"].includes(kind),"diff_line");
      if(kind===" "||kind==="+"){ensure(lines[cursor]===text,"diff_exact_line");cursor++;}
      if(kind===" "||kind==="-")out.push(text);
    }
  }
  ensure(hunks===BINDING.diff_hunks,"diff_hunk_count");out.push(...lines.slice(cursor));return out.join("");
}
let CANDIDATE,BASELINE,SCRIPT,COLLECTORS;
function bindSource(){
  CANDIDATE=decode(BINDING.candidate_b64);BASELINE=decode(BINDING.baseline_b64);
  const diff=decode(BINDING.diff_b64);
  ensure(Buffer.byteLength(CANDIDATE)===33701&&sha(CANDIDATE)===BINDING.candidate_sha256,
    "candidate_identity");
  ensure(Buffer.byteLength(BASELINE)===32502&&sha(BASELINE)===BINDING.baseline_sha256,
    "baseline_identity");
  ensure(Buffer.byteLength(diff)===2405&&sha(diff)===BINDING.diff_sha256,"diff_identity");
  let restored=CANDIDATE;
  for(const e of [...BINDING.edits].reverse()){
    ensure(restored.split(e.next).length===2,"edit_unique");
    restored=restored.replace(e.next,e.old);
  }
  ensure(restored===BASELINE&&inverseDiff(CANDIDATE,diff)===BASELINE,"baseline_inverse");
  COLLECTORS=Object.fromEntries(Object.entries(BINDING.collectors).map(([name,v])=>{
    const source=decode(v.b64);
    ensure(Buffer.byteLength(source)===v.bytes&&sha(source)===v.sha256,"collector_identity");
    const prefix="const "+name+"=";
    const line=CANDIDATE.split("\n").find(x=>x.startsWith(prefix));
    ensure(typeof line==="string"&&line.endsWith(";")&&
      JSON.parse(line.slice(prefix.length,-1))===source,"collector_in_candidate");
    return [name,source];
  }));
  ensure(Object.keys(COLLECTORS).sort().join(",")==="CLOCK,MARKER,PERSIST,READBACK,STOP",
    "collector_set");
  SCRIPT=new vm.Script("(async function(){\n"+CANDIDATE+"\n})()",
    {filename:"reviewed-d01-memory-cell.js",displayErrors:false});
  result.source={candidate_sha256:sha(CANDIDATE),candidate_bytes:Buffer.byteLength(CANDIDATE),
    baseline_sha256:sha(BASELINE),baseline_bytes:Buffer.byteLength(BASELINE),
    diff_sha256:sha(diff),logic_sha256:BINDING.logic_sha256,full_inverse:true};
}
function event(value){return JSON.stringify(value)+"\n";}
function publicPage(kind,ref){
  const nodes=kind==="after"?
    [{role:"heading",name:"Protected application access"},
     {role:"statictext",name:"Signed in. Protected application access is available."}]:
    kind==="before"?[{role:"heading",name:"Local demo"},{role:"statictext",name:"Sign in required."}]:
    kind==="application"?[{role:"heading",name:"Local demo"},
      {role:"statictext",name:"Use Sign in to open this local application."},
      {role:"button",name:"Sign in"}]:
    [{role:"textbox",name:"Username"},{role:"textbox",name:"Password"},
     {role:"button",name:"Sign in"}];
  const base=Number(ref.split(":")[1]),prefix=ref.split(":")[0]+":";
  const refs=kind==="after"||kind==="before"?[]:
    kind==="application"?[{role:"button",name:"Sign in",ref,actions:["click"]}]:[
      {role:"textbox",name:"Username",ref,actions:["type"]},
      {role:"textbox",name:"Password",ref:prefix+String(base+1000),actions:["type"]},
      {role:"button",name:"Sign in",ref:prefix+String(base+2000),actions:["click"]}
    ];
  return {structuredContent:{status:"ok",snapshot:{complete:true},content_refs:nodes,refs}};
}
function makeState(options={}){
  const own={runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,
    phase:"browser_decision",prepared_gate:true,fixture_ready:true,listener_pid_proof:true,
    fresh_paths_preflight:true,guard_pid:11,server_pid:12,browser_pid:13,helper_pid:14,
    exec_session:15,start_epoch_ms:1000000,workspace:"/synthetic/workspace",
    session:"synthetic-session",target_id:"synthetic-target",tab_id:"synthetic-tab",
    lab:"/synthetic/lab",outer_out:"/synthetic/outer",event_out:"/synthetic/event",
    cleanup_out:"/synthetic/cleanup",fresh_refs:["p1:1"],next_decision:null};
  if(options.entry){own.phase="prepared_entry";own.helper_pid=null;
    own.fixture_ready=false;own.listener_pid_proof=false;}
  const s={map:new Map([[OWN,clone(own)]]),now:own.start_epoch_ms,queue:[],
    outputs:[],metadata:[],trace:[],snapshotQueue:[],inputKinds:[],clearEvents:[],
    stopCount:0,stopProofs:[],pollCount:0,lastPollReturn:0,lastReceipt:0,
    receiptReadyProofs:0,refCounter:1,cleanup:false,options,firstHostFault:null};
  if(options.password)s.map.set(PW,PASSWORD);
  return s;
}
function trace(s,kind,data={}){
  clockGuard();ensure(s.trace.length<MAX_TRACE,"trace_cap");
  result.stub_counts[kind]=(result.stub_counts[kind]||0)+1;
  ensure(Object.values(result.stub_counts).reduce((a,b)=>a+b,0)<=MAX_CALLS,"call_cap");
  const entry={kind,...data};privacy(entry);s.trace.push(entry);return s.trace.length;
}
function store(s,key,value){
  ensure([OWN,OBS,PW].includes(key),"store_key");
  if(key!==PW)privacy(value);
  if(key===PW){ensure(value===null||value===PASSWORD,"password_store_value");
    if(value===null)s.clearEvents.push(s.trace.length);}
  if(key===OBS){
    const old=s.map.get(OBS),n=value.controller_observations.length;
    if(n>(old?.controller_observations.length??0)){
      ensure(s.trace.length<MAX_TRACE,"trace_cap");
      s.lastReceipt=s.trace.length+1;
      s.trace.push({kind:"retained_controller_receipt"});
      ensure(s.lastReceipt>s.lastPollReturn,"receipt_precedes_comparison");
    }
  }
  if(key===OWN&&value.fixture_ready===true&&s.map.get(OWN)?.fixture_ready!==true){
    ensure(s.lastReceipt>s.lastPollReturn&&s.lastReceipt>0,"ready_receipt_order");
    s.receiptReadyProofs++;
  }
  s.map.set(key,clone(value));
}
function load(s,key){ensure([OWN,OBS,PW].includes(key),"load_key");return clone(s.map.get(key));}
function decision(s,kind,ref){
  const o=load(s,OWN);o.next_decision={kind,expected_role:"textbox",expected_label:"Username"};
  if(ref!==undefined)o.next_decision.ref=ref;
  s.map.set(OWN,o);
}
function quotedArgs(cmd){
  ensure(typeof cmd==="string"&&cmd.startsWith("python3 -c "),"collector_command");
  let i=11,out=[];
  while(i<cmd.length){
    ensure(cmd[i]==="'","collector_quoting");i++;let text="";
    for(;;){
      ensure(i<cmd.length,"collector_quoting");
      if(cmd.startsWith("'\\''",i)){text+="'";i+=4;continue;}
      if(cmd[i]==="'"){i++;break;}text+=cmd[i++];
    }
    out.push(text);
    if(i===cmd.length)break;
    ensure(cmd[i]===" ","collector_quoting");i++;
  }
  return out;
}
async function local(s,args){
  const argv=quotedArgs(args.cmd),source=argv.shift();
  const label=Object.keys(COLLECTORS).find(k=>COLLECTORS[k]===source);
  ensure(label!==undefined&&args.workdir==="/synthetic/workspace","collector_allowlist");
  trace(s,"collector_"+label.toLowerCase());
  const o=load(s,OWN),rec=load(s,OBS),clock={
    monotonic_ns:String(BigInt(s.now)*1000000n),wall_epoch_ns:String(BigInt(s.now)*1000000n)};
  let data;
  if(label==="CLOCK"){ensure(argv.length===0,"collector_arguments");data=clock;}
  else if(label==="MARKER"){
    ensure(argv.length===3&&argv[0]===o.lab&&argv[1]==="11"&&argv[2]==="12","marker_bound");
    data={prepared_marker_written:true};
  } else if(label==="STOP"){
    ensure(argv.length===1,"collector_arguments");const c=JSON.parse(argv[0]);privacy(c);
    ensure(load(s,PW)===null,"clear_before_cleanup_await");
    ensure(c.first_failure===rec.first_failure&&
      c.first_observation_wall_ms===rec.first_observation_wall_ms,"latch_before_cleanup");
    s.stopProofs.push(true);s.stopCount++;s.cleanup=true;
    await Promise.resolve();ensure(load(s,PW)===null,"clear_survives_cleanup_await");
    data={clock,event_state:"written",marker_state:"stop_requested"};
  } else if(label==="READBACK"){
    ensure(argv.length===1,"collector_arguments");const c=JSON.parse(argv[0]);privacy(c);
    ensure(c.lab===o.lab&&c.server_pid===12&&c.helper_pid===o.helper_pid,"readback_bound");
    const names=["whoami","discovery","confidential_client_create","operator_login","server","maintenance_init"];
    const pids=[21,22,23,24,12,25];if(o.helper_pid!==null){names.push("helper");pids.push(14);}
    data={clock,owned_pids:Object.fromEntries(c.owned_pids.map(p=>[String(p),true])),
      ports:{"9000":true,"3000":true},lab_absent:!s.options.absence_missing,
      owned_child_exits:names.map((name,i)=>({name,pid:pids[i],exit:0})),
      helper_pid:o.helper_pid,unexpected_failure_observation:{observed:false,diagnostic:null},
      unexpected_observation_projection:"valid"};
  } else {
    ensure(argv.length===2&&argv[0]===o.cleanup_out,"persist_bound");
    const record=JSON.parse(argv[1]);privacy(record);
    ensure(Buffer.byteLength(JSON.stringify(record))<=262144,"metadata_cap");s.metadata.push(clone(record));
    if(s.options.metadata_pending)return {session_id:99,output:""};
    data={written_exclusive:true};
  }
  return {exit_code:0,output:JSON.stringify(data)};
}
function bound(s,args,ref=false){
  const o=load(s,OWN);ensure(args.session===o.session&&args.target_id===o.target_id&&
    args.tab_id===o.tab_id,"bound_browser_handles");
  if(ref)ensure(typeof args.ref==="string"&&/^p[0-9]+:[0-9]+$/.test(args.ref),"bound_reference");
}
async function poll(s,args){
  ensure(args.session_id===15&&args.chars==="","bound_controller");
  trace(s,"controller_poll",{cleanup:s.cleanup});s.pollCount++;
  let r;
  if(s.cleanup){
    r={exit_code:0,output:event(s.options.cleanup_helper_error?
      {helper_completed:true,exit:1,fixture_finished:true}:{fixture_finished:true})};
  } else {
    const next=s.queue.shift()??{};
    if(next.throw){throw new Error(EXCEPTION);}
    if(next.at!==undefined)s.now=1000000+next.at;
    r={output:next.output??"",...(next.exit===undefined?{}:{exit_code:next.exit})};
  }
  s.lastPollReturn=s.trace.length;return r;
}
function bridge(s,operation){
  const remember=e=>{if(FAULTS.has(e)&&s.firstHostFault===null)s.firstHostFault=e;throw e;};
  try{const v=operation();return v instanceof Promise?v.catch(remember):v;}
  catch(e){return remember(e);}
}
function tools(s){
  const methods={
    exec_command:a=>local(s,a),write_stdin:a=>poll(s,a),
    mcp__cua_driver__browser_navigate:async a=>{
      bound(s,a);ensure(["http://localhost:3000/","http://localhost:3000/protected"].includes(a.url),
        "navigation_allowlist");trace(s,"navigate");return {structuredContent:{status:"ok"}};
    },
    mcp__cua_driver__get_browser_state:async a=>{
      bound(s,a);ensure(a.snapshot_format==="semantic_v2"&&a.include_screenshot===false,
        "snapshot_public_only");trace(s,"snapshot");
      const kind=s.snapshotQueue.shift()??"generic";const ref="p1:"+String(++s.refCounter);
      if(kind!==null&&typeof kind==="object")return clone(kind);
      if(kind==="missing")return {structuredContent:{status:"ok",snapshot:{complete:true},
        content_refs:[],refs:[]}};
      return publicPage(kind,ref);
    },
    mcp__cua_driver__browser_click:async a=>{
      bound(s,a,true);ensure(a.input_route==="dom_event","click_route");
      if(s.options.projection_protocol)ensure(load(s,OWN).fresh_refs.length===0&&
        a.ref===s.options.expected_action_ref,"projection_single_use_before_await");
      trace(s,"click");s.inputKinds.push("click");return {structuredContent:{effect:"confirmed"}};
    },
    mcp__cua_driver__browser_type:async a=>{
      bound(s,a,true);ensure(a.replace===true,"type_replace");
      const kind=a.text===PASSWORD?"password":a.text==="admin"?"username":null;
      ensure(kind!==null,"type_value");trace(s,"type",{kind,password_matches:kind==="password"});
      s.inputKinds.push(kind);return {structuredContent:{effect:"confirmed"}};
    },
    mcp__cua_driver__kill_app:async a=>{
      ensure(a.pid===13,"kill_bound_owned_pid");trace(s,"kill");
      if(s.options.driver_exception)throw new Error(EXCEPTION);
      return {structuredContent:{status:"ok"}};
    },
    mcp__cua_driver__end_session:async a=>{
      ensure(a.session==="synthetic-session","end_bound_session");trace(s,"end_session");
      return {structuredContent:{active:false,session:a.session}};
    },
    mcp__cua_driver__list_windows:async a=>{
      ensure(a.pid===13,"windows_bound_pid");trace(s,"windows");
      return {structuredContent:{windows:[]}};
    }
  };
  return Object.freeze(Object.fromEntries(Object.entries(methods)
    .map(([name,f])=>[name,a=>bridge(s,()=>f(a))])));
}
async function cell(s){
  clockGuard();ensure(result.cells<MAX_CELLS,"cell_cap");result.cells++;
  const sandbox=Object.create(null);
  Object.assign(sandbox,{tools:tools(s),store:(k,v)=>bridge(s,()=>store(s,k,v)),
    load:k=>bridge(s,()=>load(s,k)),text:v=>bridge(s,()=>{privacy(v);s.outputs.push(clone(v));}),
    exit:()=>{throw EXIT;},Date:Object.freeze({now:()=>s.now})});
  const ctx=vm.createContext(sandbox,{codeGeneration:{strings:false,wasm:false}});
  let timer;
  try{
    const remaining=Math.max(1,Math.floor(LIMIT_MS-performance.now()));
    const execution=SCRIPT.runInContext(ctx,{timeout:remaining,displayErrors:false});
    await Promise.race([Promise.resolve(execution),new Promise((_,reject)=>{
      timer=setTimeout(()=>reject(failure("real_deadline")),remaining);
    })]);
  }catch(e){if(e!==EXIT)throw e;}
  finally{if(timer!==undefined)clearTimeout(timer);}
  clockGuard();if(s.firstHostFault!==null)throw s.firstHostFault;
  ensure(s.outputs.length>0,"cell_output");
  for(const [k,v] of s.map)if(k!==PW)privacy(v);
  privacy(s.outputs);privacy(s.metadata);privacy(s.trace);
  return s.outputs.at(-1);
}
function finalOutcome(s,expected,release){
  const o=s.outputs.at(-1);privacy(o);
  ensure(o.whole_cleanup_within60_proven===false,"no_whole60_credit");
  ensure(o.journey_credit!==true,"no_journey_credit");
  if(expected===null)ensure(o.result==="browser_decision_observed_only","observed_only");
  else ensure(o.result==="failed"&&o.first_failure===expected,"first_failure_matches");
  if(release!==undefined)ensure(o.resource_release_proven===release,"release_separate");
  if(s.stopCount){ensure(s.stopCount===1&&s.stopProofs.length===1,"single_cleanup_protocol");
    ensure(load(s,PW)===null,"password_cleared");}
}
function firstActive(s,output="",extra={}){s.queue.push({output,...extra});}
function setTime(s,ms){s.now=1000000+ms;}
function partialEvent(length){
  const fixed={unexpected_failure_observation:{observed:true,diagnostic:null},
    unexpected_observation_projection:"valid",opaque:PARTIAL,pad:""};
  let line=event(fixed);if(length!==undefined){fixed.pad="x".repeat(length-line.length);line=event(fixed);}
  ensure(length===undefined||line.length===length,"fixture_line_size");
  return line;
}
const PUBLIC_PAIRS=[
  ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
  ["heading","Local demo wants to use your riAuth account"],
  ["heading","Continue to Local demo?"],["heading","Protected application access"],
  ["button","Sign in"],["button","Allow"],["button","Continue"],
  ["textbox","Username"],["textbox","Password"],
  ["textbox","Authenticator or recovery code (if enabled)"],
  ["statictext","Signed in. Protected application access is available."]
];
function projectionNode(role,name){
  return {role,name,value:PROJECTION,url:PROJECTION,subject:PROJECTION,
    state:PROJECTION,cookie:PROJECTION,headers:{authorization:PROJECTION},error:PROJECTION};
}
function projectionRef(role,name,ref,actions){
  return {...projectionNode(role,name),ref,actions};
}
function projectionSnapshot(nodes,refs,changes={}){
  return {structuredContent:{status:"ok",snapshot:{complete:true},
    content_refs:nodes,refs,...changes}};
}
function projectionDecision(s,kind,role,name,ref){
  decision(s,kind,ref);const o=load(s,OWN);
  o.next_decision.expected_role=role;o.next_decision.expected_label=name;s.map.set(OWN,o);
}
function keysExactly(value,keys){
  return value!==null&&typeof value==="object"&&!Array.isArray(value)&&
    Object.keys(value).sort().join(",")===[...keys].sort().join(",");
}
function projectionShape(value){
  ensure(keysExactly(value,["observed","refs"])&&
    Array.isArray(value.observed)&&Array.isArray(value.refs),"projection_closed_shape");
  for(const row of value.observed){
    ensure(keysExactly(row,["role","name"])&&PUBLIC_PAIRS.some(([role,name])=>
      row.role===role&&row.name===name),"projection_canonical_pair");
  }
  for(const row of value.refs){
    ensure(keysExactly(row,["role","name","action","ref"])&&PUBLIC_PAIRS.some(([role,name])=>
      row.role===role&&row.name===name)&&typeof row.ref==="string"&&
      /^p[0-9]+:[0-9]+$/.test(row.ref),"projection_canonical_ref");
    ensure((row.role==="button"&&row.action==="click")||
      (row.role==="textbox"&&["Username","Password"].includes(row.name)&&row.action==="type"),
      "projection_action_pair");
  }
  privacy(value);
}
function projected(s,out,expected){
  finalOutcome(s,null,false);projectionShape(out.public_decision);
  const owned=load(s,OWN);projectionShape(owned.public_decision);
  ensure(JSON.stringify(out.public_decision)===JSON.stringify(expected)&&
    JSON.stringify(owned.public_decision)===JSON.stringify(expected),"projection_exact_output_and_store");
  return out;
}
function projectionFailed(s,out,expected,emptyOwned=true){
  finalOutcome(s,expected,true);
  ensure(!Object.hasOwn(out,"public_decision")&&s.inputKinds.length===0,
    "projection_failure_no_public_decision_or_input");
  if(emptyOwned)ensure(JSON.stringify(load(s,OWN).public_decision)===
    '{"observed":[],"refs":[]}',"projection_processed_failure_empty");
}
function noProjectionInput(s){
  ensure(s.inputKinds.length===0&&!s.trace.some(x=>["click","type"].includes(x.kind)),
    "projection_no_automatic_input");
}

async function runCase(name){
  let s;
  switch(name){
    case "phase_entry_then_continuation":{
      s=makeState({entry:true});s.snapshotQueue=["before","application"];
      firstActive(s,event({fixture_ready:true,guard_pid:11,server_pid:12,helper_pid:14,lab:"/synthetic/lab"}));
      await cell(s);ensure(load(s,OWN).phase==="browser_decision"&&load(s,OWN).helper_pid===14,
        "entry_phase_and_helper");ensure(s.receiptReadyProofs===1,"ready_receipt_proved");
      const start=load(s,OWN).start_epoch_ms;decision(s,"snapshot");await cell(s);
      ensure(load(s,OWN).start_epoch_ms===start,"startup_budget_retained");finalOutcome(s,null,false);break;
    }
    case "password_survives_until_password_dispatch":{
      s=makeState({password:true});
      for(const kind of ["username","click","snapshot"]){
        const ref=load(s,OWN).fresh_refs[0];decision(s,kind,kind==="snapshot"?undefined:ref);
        await cell(s);ensure(load(s,PW)===PASSWORD&&s.clearEvents.length===0,"secret_survives_nonpassword");
      }
      decision(s,"password",load(s,OWN).fresh_refs[0]);await cell(s);
      ensure(s.inputKinds.join(",")==="username,click,password","input_route_sequence");
      ensure(load(s,PW)===null&&s.clearEvents.length===1,"password_dispatch_consumes_once");
      finalOutcome(s,null,false);break;
    }
    case "missing_password_refuses_without_input":{
      s=makeState();decision(s,"password","p1:1");await cell(s);
      ensure(s.inputKinds.length===0,"no_input_on_refusal");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "unowned_context_refuses_without_operations":{
      s=makeState();const own=load(s,OWN);own.driver_owned=false;s.map.set(OWN,own);
      await cell(s);ensure(s.trace.length===0&&s.outputs.at(-1).proposal_refused===
        "fresh_owned_context_required","unowned_refusal");break;
    }
    case "undeclared_ref_refuses_input":{
      s=makeState({password:true});decision(s,"click","p1:999");await cell(s);
      ensure(s.inputKinds.length===0,"no_input_on_refusal");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "stale_ref_cannot_cross_cells":{
      s=makeState();decision(s,"click","p1:1");await cell(s);
      ensure(!load(s,OWN).fresh_refs.includes("p1:1"),"fresh_ref_replaces_consumed");
      decision(s,"click","p1:1");await cell(s);
      ensure(s.inputKinds.length===1,"single_use_ref");finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "invalid_kind_repeated_cleanup_clears_before_await":{
      s=makeState({password:true});decision(s,"not-declared");await cell(s);
      ensure(s.clearEvents.length===2&&s.stopCount===1,"repeat_clear_no_repeat_await");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "helper_zero_final_snapshot_then_cleanup":
    case "helper_zero_missing_page_still_fails":
    case "helper_nonzero_final_still_refuses":
    case "helper_zero_other_kind_still_refuses":{
      s=makeState();
      const other=name==="helper_zero_other_kind_still_refuses";
      const nonzero=name==="helper_nonzero_final_still_refuses";
      decision(s,other?"click":"protected_after",other?"p1:1":undefined);
      s.snapshotQueue=[name==="helper_zero_missing_page_still_fails"?"missing":"after"];
      firstActive(s,event({helper_completed:true,exit:nonzero?1:0}));await cell(s);
      const snapshots=s.trace.filter(x=>x.kind==="snapshot").length;
      ensure(s.inputKinds.length===0&&!s.trace.some(x=>x.kind==="navigate"),"read_only_handoff");
      if(nonzero||other){ensure(snapshots===0,"zero_only_declared_final");
        finalOutcome(s,nonzero?"helper_failed":"helper_completed_before_app_checkpoint",true);}
      else {ensure(snapshots===1,"one_final_snapshot");
        if(name==="helper_zero_missing_page_still_fails")finalOutcome(s,"browser_decision_unconfirmed",true);
        else {ensure(load(s,OBS).protected_after_page===true,"final_page_proved");finalOutcome(s,null,true);}}
      break;
    }
    case "helper_zero_prepared_phase_still_refuses":{
      s=makeState({entry:true});decision(s,"protected_after");
      firstActive(s,event({fixture_ready:true,guard_pid:11,server_pid:12,helper_pid:14,
        lab:"/synthetic/lab",helper_completed:true,exit:0}));await cell(s);
      ensure(!s.trace.some(x=>["navigate","snapshot","type","click"].includes(x.kind)),"prepared_phase_refusal");
      finalOutcome(s,"helper_completed_before_app_checkpoint",true);break;
    }
    case "partial_line_drains_before_output_and_next_cell":
    case "partial_exact_cap_is_accepted":{
      s=makeState();decision(s,"snapshot");
      const line=partialEvent(name==="partial_exact_cap_is_accepted"?16384:undefined);
      firstActive(s);firstActive(s,line.slice(0,-1));firstActive(s,line.slice(-1));await cell(s);
      ensure(s.pollCount===3&&load(s,OBS).unexpected_failure_observation?.observed===true,
        "partial_event_processed_before_output");
      ensure([...s.map.keys()].sort().join(",")===[OWN,OBS].sort().join(","),"no_partial_store");
      finalOutcome(s,null,false);
      if(name==="partial_line_drains_before_output_and_next_cell"){
        const previous=s.pollCount;decision(s,"snapshot");await cell(s);
        ensure(s.pollCount-previous===2,"no_partial_carry_next_cell");finalOutcome(s,null,false);
      }break;
    }
    case "partial_over_cap_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s);firstActive(s,partialEvent(16385));
      await cell(s);finalOutcome(s,"controller_observation_invalid",true);break;
    }
    case "partial_joined_controller_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s);firstActive(s,'{"opaque":"'+PARTIAL,{exit:0});
      await cell(s);finalOutcome(s,"controller_completed_before_app_checkpoint",true);break;
    }
    case "partial_unavailable_controller_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s,'{"opaque":"'+PARTIAL);
      s.queue.push({throw:true});await cell(s);
      finalOutcome(s,"controller_observation_unavailable",false);break;
    }
    case "partial_deadline_prevents_extra_poll":{
      s=makeState();decision(s,"snapshot");firstActive(s);
      firstActive(s,'{"opaque":"'+PARTIAL,{at:840000});await cell(s);
      ensure(s.pollCount===3&&s.trace.filter(x=>x.kind==="controller_poll"&&!x.cleanup).length===2,
        "no_extra_active_poll_at_deadline");
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "partial_completed_late_still_refuses":{
      s=makeState();decision(s,"snapshot");const line=partialEvent();
      firstActive(s);firstActive(s,line.slice(0,-1),{at:839999});
      firstActive(s,line.slice(-1),{at:840000});await cell(s);
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "retained_start_budget_refuses_next_cell":{
      s=makeState();decision(s,"snapshot");await cell(s);const o=load(s,OWN);
      setTime(s,840000);decision(s,"snapshot");const before=s.trace.filter(x=>x.kind==="snapshot").length;
      await cell(s);ensure(load(s,OWN).start_epoch_ms===o.start_epoch_ms,"startup_budget_retained");
      ensure(s.trace.filter(x=>x.kind==="snapshot").length===before,"no_snapshot_after_deadline");
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "inclusive_cleanup_deadline_does_not_invent_join":{
      s=makeState();decision(s,"snapshot");setTime(s,900000);await cell(s);
      ensure(load(s,OBS).controller_exit===null,"no_join_credit_at_inclusive_limit");
      finalOutcome(s,"browser_active_deadline",false);break;
    }
    case "first_page_failure_survives_later_helper_error":
    case "missing_absence_proof_prevents_release":
    case "cleanup_exception_is_private":
    case "pending_metadata_command_prevents_release":{
      s=makeState({cleanup_helper_error:name==="first_page_failure_survives_later_helper_error",
        absence_missing:name==="missing_absence_proof_prevents_release",
        driver_exception:name==="cleanup_exception_is_private",
        metadata_pending:name==="pending_metadata_command_prevents_release",password:true});
      decision(s,"protected_after");s.snapshotQueue=["missing"];await cell(s);
      finalOutcome(s,"browser_decision_unconfirmed",
        name!=="missing_absence_proof_prevents_release"&&name!=="pending_metadata_command_prevents_release");
      if(name==="cleanup_exception_is_private")ensure(load(s,OBS).cleanup_errors.includes(
        "driver_operation_unconfirmed"),"driver_failure_retained");
      break;
    }
    case "projection_exact_pairs_and_private_field_omission":{
      s=makeState();projectionDecision(s,"snapshot","heading","Local demo");
      const nodes=PUBLIC_PAIRS.map(([role,name])=>projectionNode(role,name));
      const refs=[
        projectionRef("button","Sign in","p70:1",["click",PROJECTION]),
        projectionRef("button","Allow","p70:2",["click"]),
        projectionRef("button","Continue","p70:3",["click"]),
        projectionRef("textbox","Username","p70:4",["type"]),
        projectionRef("textbox","Password","p70:5",["type"]),
        projectionRef("textbox","Authenticator or recovery code (if enabled)","p70:6",["type","click"]),
        projectionRef("heading","Local demo","p70:7",["click"])
      ];
      nodes.push(projectionNode("textbox",PROJECTION));
      s.snapshotQueue=[projectionSnapshot(nodes,refs,{page:{url:PROJECTION},account:{subject:PROJECTION},
        error:PROJECTION,headers:{authorization:PROJECTION},cookie:PROJECTION,state:PROJECTION})];
      const out=await cell(s);
      projected(s,out,{observed:PUBLIC_PAIRS.map(([role,name])=>({role,name})),refs:[
        {role:"button",name:"Sign in",action:"click",ref:"p70:1"},
        {role:"button",name:"Allow",action:"click",ref:"p70:2"},
        {role:"button",name:"Continue",action:"click",ref:"p70:3"},
        {role:"textbox",name:"Username",action:"type",ref:"p70:4"},
        {role:"textbox",name:"Password",action:"type",ref:"p70:5"}
      ]});noProjectionInput(s);break;
    }
    case "projection_otp_observed_without_action_or_value":{
      s=makeState();projectionDecision(s,"snapshot","textbox","Authenticator or recovery code (if enabled)");
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("textbox","Authenticator or recovery code (if enabled)")],[
        projectionRef("textbox","Authenticator or recovery code (if enabled)","p71:1",["type","click"])])];
      projected(s,await cell(s),{observed:[
        {role:"textbox",name:"Authenticator or recovery code (if enabled)"}],refs:[]});
      noProjectionInput(s);break;
    }
    case "projection_required_consent_allow_roundtrip":
    case "projection_optional_consent_continue_roundtrip":{
      const required=name==="projection_required_consent_allow_roundtrip";
      const heading=required?"Local demo wants to use your riAuth account":"Continue to Local demo?";
      const label=required?"Allow":"Continue",ref=required?"p72:1":"p73:1";
      s=makeState({projection_protocol:true,expected_action_ref:ref});
      projectionDecision(s,"snapshot","heading",heading);
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("heading",heading),projectionNode("button",label)],[
        projectionRef("button",label,ref,["click"])])];
      projected(s,await cell(s),{observed:[{role:"heading",name:heading},{role:"button",name:label}],
        refs:[{role:"button",name:label,action:"click",ref}]});
      noProjectionInput(s);
      projectionDecision(s,"click","textbox","Username",ref);
      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
        projectionRef("textbox","Username","p74:1",["type"])])];
      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
        refs:[{role:"textbox",name:"Username",action:"type",ref:"p74:1"}]});
      ensure(s.inputKinds.join(",")==="click"&&!load(s,OWN).fresh_refs.includes(ref),
        "projection_consent_dispatch_and_freshness");break;
    }
    case "projection_role_and_label_rejections":{
      for(const pair of [["heading","Allow"],["button","Allow "+PROJECTION]]){
        s=makeState();projectionDecision(s,"snapshot","button","Allow");
        s.snapshotQueue=[projectionSnapshot([projectionNode(...pair)],[
          projectionRef(...pair,"p75:1",["click"])])];
        projectionFailed(s,await cell(s),"browser_decision_unconfirmed");
      }break;
    }
    case "projection_incomplete_and_error_rejections":{
      for(const variant of ["incomplete","fixed_error","bad_status","tool_error"]){
        s=makeState();projectionDecision(s,"snapshot","textbox","Username");
        const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
          refs:[{role:"button",name:"Allow",action:"click",ref:"p76:9"}]};s.map.set(OWN,owned);
        const raw=projectionSnapshot([projectionNode("textbox","Username")],[
          projectionRef("textbox","Username","p76:1",["type"])]);
        if(variant==="incomplete")raw.structuredContent.snapshot.complete=false;
        if(variant==="fixed_error")raw.structuredContent.content_refs.push(
          projectionNode("statictext","Local demo could not complete this request."));
        if(variant==="bad_status")raw.structuredContent.status="refused";
        if(variant==="tool_error"){raw.isError=true;raw.error=PROJECTION;}
        s.snapshotQueue=[raw];projectionFailed(s,await cell(s),
          variant==="tool_error"?"browser_tool_refused":"browser_decision_unconfirmed",
          variant!=="tool_error");
      }break;
    }
    case "projection_missing_or_wrong_advertised_action":{
      s=makeState();projectionDecision(s,"snapshot","textbox","Username");
      const missing=projectionRef("textbox","Password","p77:2",["type"]);delete missing.actions;
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("textbox","Username"),projectionNode("textbox","Password"),
        projectionNode("button","Sign in"),projectionNode("button","Allow")],[
        projectionRef("textbox","Username","p77:1",["click"]),missing,
        projectionRef("button","Sign in","p77:3",["type"]),
        projectionRef("button","Allow","p77:4",["click",PROJECTION])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Sign in"},{role:"button",name:"Allow"},
        {role:"textbox",name:"Username"},{role:"textbox",name:"Password"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p77:4"}]});
      noProjectionInput(s);break;
    }
    case "projection_malformed_refs":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        projectionRef("button","Allow","x1:1",["click"]),
        projectionRef("button","Allow","p1:-1",["click"]),
        projectionRef("button","Allow",PROJECTION,["click"]),
        projectionRef("button","Allow",17,["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
      noProjectionInput(s);break;
    }
    case "projection_disjoint_rows_do_not_infer_association":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p78:9"}]};s.map.set(OWN,owned);
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        {ref:"p78:1",actions:["click"],value:PROJECTION},
        projectionRef("button","Unrelated "+PROJECTION,"p78:2",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
      ensure(!load(s,OWN).fresh_refs.includes("p78:9"),"projection_no_cross_snapshot_join");
      noProjectionInput(s);break;
    }
    case "projection_duplicate_actions_preserve_multiplicity":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      const row=projectionRef("button","Allow","p79:1",["click"]);
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("button","Allow"),projectionNode("button","Allow")],[
        row,clone(row),projectionRef("button","Allow","p79:2",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[
        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
        {role:"button",name:"Allow",action:"click",ref:"p79:2"}]});
      noProjectionInput(s);break;
    }
    case "projection_latest_success_replaces_previous_snapshot":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        projectionRef("button","Allow","p80:1",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p80:1"}]});
      projectionDecision(s,"snapshot","textbox","Username");
      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
        projectionRef("textbox","Username","p81:1",["type"])])];
      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
        refs:[{role:"textbox",name:"Username",action:"type",ref:"p81:1"}]});
      ensure(!load(s,OWN).fresh_refs.includes("p80:1"),"projection_latest_ref_only");
      projectionDecision(s,"click","textbox","Username","p80:1");
      await cell(s);finalOutcome(s,"browser_decision_unconfirmed",true);
      ensure(s.inputKinds.length===0,
        "projection_old_ref_refused");break;
    }
    default:throw failure("unknown_case");
  }
  privacy(s.outputs);privacy(s.metadata);privacy(s.trace);
}
async function main(){
  let current=null;
  try{
    clockGuard();bindSource();
    ensure(BINDING.case_plan.length===new Set(BINDING.case_plan.map(x=>x.name)).size,"unique_case_names");
    for(const row of BINDING.case_plan){
      clockGuard();current=row.name;result.attempted.push(row.name);
      await runCase(row.name);result.completed.push(row.name);
      result.completed_groups[row.group]=(result.completed_groups[row.group]||0)+1;
    }
    ensure(result.cells===51,"projection_complete_cell_count");
  }catch(e){
    const code=(e!==null&&(typeof e==="object"||typeof e==="function")&&FAULTS.has(e))?
      FAULTS.get(e):"unexpected";
    result.first_failure={case:current,check:BINDING.check_names.includes(code)?code:"unexpected"};
  }
  result.unreached=BINDING.case_plan.map(x=>x.name).filter(x=>!result.attempted.includes(x));
  result.elapsed_ms=performance.now();
  if(result.elapsed_ms>=LIMIT_MS&&result.first_failure===null)
    result.first_failure={case:null,check:"real_deadline"};
  // Only the closed finite packet is written; no raw exception/trace/source/stub value.
  const output=JSON.stringify(result);
  if(PRIVATE.some(x=>output.includes(x))){
    process.stdout.write(JSON.stringify({schema:result.schema,first_failure:{case:null,check:"privacy_sink"}})+"\n");
    process.exitCode=1;return;
  }
  process.stdout.write(output+"\n");process.exitCode=result.first_failure===null?0:1;
}
await main();
```

### Complete serialized future payload — UNEXECUTED

```javascript
import vm from "node:vm";
import {createHash} from "node:crypto";
import {performance} from "node:perf_hooks";
const BINDING = {
  "candidate_b64": "Ly8gUHJvc3BlY3RpdmUgT05FIGZ1bmN0aW9ucy5leGVjIGNlbGw7IE5PVCBleGVjdXRlZCBpbiB0aGlzIGRlc2lnbiBwaGFzZS4KY29uc3QgQ0xPQ0s9ImltcG9ydCBqc29uLHRpbWVcbnByaW50KGpzb24uZHVtcHMoeydtb25vdG9uaWNfbnMnOnN0cih0aW1lLm1vbm90b25pY19ucygpKSwnd2FsbF9lcG9jaF9ucyc6c3RyKHRpbWUudGltZV9ucygpKX0pKVxuIjsKY29uc3QgU1RPUD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN5cyx0aW1lXG5jPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMV0pXG5jbG9jaz17J21vbm90b25pY19ucyc6c3RyKHRpbWUubW9ub3RvbmljX25zKCkpLCd3YWxsX2Vwb2NoX25zJzpzdHIodGltZS50aW1lX25zKCkpfVxucmVjb3JkPXsnc2NoZW1hJzoncmlhdXRoLmQwMS1maXJzdC1vYnNlcnZhdGlvbi92MScsJ2ZpcnN0X2ZhaWx1cmUnOmNbJ2ZpcnN0X2ZhaWx1cmUnXSwnZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyc6Y1snZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyddLCdmaXJzdF9ldmVudF9wcm92ZW4nOkZhbHNlLCdjbG9jayc6Y2xvY2t9XG5ldmVudF9zdGF0ZT0nd3JpdGVfdW5jb25maXJtZWQnXG50cnk6XG4gICAgZmQ9b3Mub3BlbihwYXRobGliLlBhdGgoY1snZXZlbnRfb3V0J10pLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApXG4gICAgd2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgICAgIGYud3JpdGUoanNvbi5kdW1wcyhyZWNvcmQsc29ydF9rZXlzPVRydWUpKydcXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSlcbiAgICBldmVudF9zdGF0ZT0nd3JpdHRlbidcbmV4Y2VwdCBPU0Vycm9yOnBhc3NcbiMgQXR0ZW1wdCBjbG9jayBwZXJzaXN0ZW5jZSBCRUZPUkUgbWFya2VyL3N0YXRlL2J1ZGdldCBjb21wYXJpc29ucy5cbiMgQSBtZXRhZGF0YSBlcnJvciBkb2VzIG5vdCBwcmV2ZW50IHRoZSBvcmlnaW5hbCBlc3NlbnRpYWwgc3RvcCBwcm90b2NvbC5cbmxhYj1wYXRobGliLlBhdGgoY1snbGFiJ10pO21hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbnRyeTpcbiAgICBpZiBsYWIuZXhpc3RzKCk6XG4gICAgICAgIGluZm89bGFiLmxzdGF0KClcbiAgICAgICAgaWYgbm90IHN0YXQuU19JU0RJUihpbmZvLnN0X21vZGUpIG9yIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpIT0wbzcwMCBvciBpbmZvLnN0X3VpZCE9b3MuZ2V0dWlkKCk6XG4gICAgICAgICAgICBtYXJrZXJfc3RhdGU9J293bmVyc2hpcF91bmtub3duJ1xuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgbWFya2VyX3N0YXRlPSdzdG9wX3JlcXVlc3RlZCdcbiAgICAgICAgICAgIGZvciBuYW1lIGluICgoJ3VpLWZhaWx1cmUnLCdzdG9wJykgaWYgY1snZmlyc3RfZmFpbHVyZSddIGlzIG5vdCBOb25lIGVsc2UgKCdzdG9wJywpKTpcbiAgICAgICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgICAgIGZkPW9zLm9wZW4obGFiL25hbWUsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbiAgICAgICAgICAgICAgICAgICAgb3MuY2xvc2UoZmQpXG4gICAgICAgICAgICAgICAgZXhjZXB0IEZpbGVOb3RGb3VuZEVycm9yOm1hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbiAgICAgICAgICAgICAgICBleGNlcHQgRmlsZUV4aXN0c0Vycm9yOnBhc3NcbmV4Y2VwdCBPU0Vycm9yOm1hcmtlcl9zdGF0ZT0nc3RvcF91bmNvbmZpcm1lZCdcbnByaW50KGpzb24uZHVtcHMoeydjbG9jayc6Y2xvY2ssJ2V2ZW50X3N0YXRlJzpldmVudF9zdGF0ZSwnbWFya2VyX3N0YXRlJzptYXJrZXJfc3RhdGV9KSlcbiI7CmNvbnN0IFJFQURCQUNLPSJpbXBvcnQganNvbixvcyxwYXRobGliLHN0YXQsc3VicHJvY2VzcyxzeXMsdGltZVxuYz1qc29uLmxvYWRzKHN5cy5hcmd2WzFdKVxuY2hpbGRyZW49Tm9uZTtoZWxwZXJfcGlkPWNbJ2hlbHBlcl9waWQnXTtvdXRlcl9vYnNlcnZhdGlvbj1Ob25lO3Byb2plY3Rpb249J3VuYXZhaWxhYmxlJ1xuZGVmIGZpbml0ZV9vYnNlcnZhdGlvbih2YWx1ZSk6XG4gICAgaWYgdHlwZSh2YWx1ZSkgaXMgbm90IGRpY3Qgb3Igc2V0KHZhbHVlKSE9IHsnb2JzZXJ2ZWQnLCdkaWFnbm9zdGljJ30gb3IgdHlwZSh2YWx1ZVsnb2JzZXJ2ZWQnXSkgaXMgbm90IGJvb2w6XG4gICAgICAgIHJldHVybiBOb25lXG4gICAgZGlhZ25vc3RpYz12YWx1ZVsnZGlhZ25vc3RpYyddXG4gICAgaWYgZGlhZ25vc3RpYyBpcyBOb25lOlxuICAgICAgICByZXR1cm4geydvYnNlcnZlZCc6dmFsdWVbJ29ic2VydmVkJ10sJ2RpYWdub3N0aWMnOk5vbmV9XG4gICAgaWYgbm90IHZhbHVlWydvYnNlcnZlZCddIG9yIHR5cGUoZGlhZ25vc3RpYykgaXMgbm90IGRpY3Qgb3Igc2V0KGRpYWdub3N0aWMpIT0geydzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnfTpcbiAgICAgICAgcmV0dXJuIE5vbmVcbiAgICBzaXRlcz0oJ2hhbmRsZXInLCdzZXJ2ZXInLCdtYWluJylcbiAgICBraW5kcz0oJ0F0dHJpYnV0ZUVycm9yJywnVHlwZUVycm9yJywnVmFsdWVFcnJvcicsJ0tleUVycm9yJywnT1NFcnJvcicsJ0Jyb2tlblBpcGVFcnJvcicsJ0Nvbm5lY3Rpb25SZXNldEVycm9yJywnVGltZW91dEVycm9yJywnb3RoZXInKVxuICAgIGZ1bmN0aW9ucz0oJ0hlYWRlclJlYWRlci5yZWFkbGluZScsJ0RlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0JywnRGVtby5iZWdpbicsJ0RlbW8uY2FsbGJhY2snLCdEZW1vLmludm9rZScsJ0hhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0JywnSGFuZGxlci5zZW5kX2Vycm9yJywnSGFuZGxlci5nZXQnLCdIYW5kbGVyLnJlcGx5JywnbWFpbicpXG4gICAgc2l0ZSxraW5kLGZ1bmN0aW9uLGxpbmU9KGRpYWdub3N0aWNba10gZm9yIGsgaW4gKCdzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnKSlcbiAgICBpZiB0eXBlKHNpdGUpIGlzIG5vdCBzdHIgb3Igc2l0ZSBub3QgaW4gc2l0ZXMgb3IgdHlwZShraW5kKSBpcyBub3Qgc3RyIG9yIGtpbmQgbm90IGluIGtpbmRzOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIGlmIG5vdCAoKGZ1bmN0aW9uIGlzIE5vbmUgYW5kIGxpbmUgaXMgTm9uZSkgb3IgKHR5cGUoZnVuY3Rpb24pIGlzIHN0ciBhbmQgZnVuY3Rpb24gaW4gZnVuY3Rpb25zIGFuZCB0eXBlKGxpbmUpIGlzIGludCBhbmQgMTw9bGluZTw9MTAyNCkpOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIHJldHVybiB7J29ic2VydmVkJzpUcnVlLCdkaWFnbm9zdGljJzp7J3NpdGUnOnNpdGUsJ2V4Y2VwdGlvbl9jbGFzcyc6a2luZCwnb3duX2Z1bmN0aW9uJzpmdW5jdGlvbiwnb3duX2xpbmUnOmxpbmV9fVxub3V0ZXI9cGF0aGxpYi5QYXRoKGNbJ291dGVyX291dCddKVxuaWYgb3V0ZXIuaXNfZmlsZSgpOlxuICAgIGZkPW9zLm9wZW4ob3V0ZXIsb3MuT19SRE9OTFl8b3MuT19OT0ZPTExPV3xvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbz1vcy5mc3RhdChmZClcbiAgICAgICAgaWYgbm90IChzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKT09MG82MDAgYW5kIDA8aW5mby5zdF9zaXplPD0yNjIxNDQpOlxuICAgICAgICAgICAgcmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIHdpdGggb3MuZmRvcGVuKGZkLCdyYicsY2xvc2VmZD1GYWxzZSkgYXMgc291cmNlOnJhd19ieXRlcz1zb3VyY2UucmVhZCgyNjIxNDUpXG4gICAgICAgIGlmIG5vdCAwPGxlbihyYXdfYnl0ZXMpPD0yNjIxNDQ6cmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIGRhdGE9anNvbi5sb2FkcyhyYXdfYnl0ZXMpXG4gICAgZmluYWxseTpvcy5jbG9zZShmZClcbiAgICBvdXRlcl9vYnNlcnZhdGlvbj1maW5pdGVfb2JzZXJ2YXRpb24oZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbicpKVxuICAgIHByb2plY3Rpb249ZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfb2JzZXJ2YXRpb25fcHJvamVjdGlvbicpXG4gICAgaWYgcHJvamVjdGlvbiBub3QgaW4gKCd1bmF2YWlsYWJsZScsJ3ZhbGlkJywnaW52YWxpZCcpOnByb2plY3Rpb249J2ludmFsaWQnXG4gICAgaWYgcHJvamVjdGlvbj09J3ZhbGlkJyBhbmQgb3V0ZXJfb2JzZXJ2YXRpb24gaXMgTm9uZTpwcm9qZWN0aW9uPSdpbnZhbGlkJ1xuICAgIGFsbG93ZWQ9eyd3aG9hbWknLCdkaXNjb3ZlcnknLCdjb25maWRlbnRpYWxfY2xpZW50X2NyZWF0ZScsJ29wZXJhdG9yX2xvZ2luJywnc2VydmVyJywnbWFpbnRlbmFuY2VfaW5pdCd9XG4gICAgc3RhcnRlZD1kYXRhLmdldCgnaGVscGVyX2ludm9jYXRpb25zJyk9PTEgYW5kIHR5cGUoZGF0YS5nZXQoJ2hlbHBlcl9waWQnKSkgaXMgaW50IGFuZCBkYXRhWydoZWxwZXJfcGlkJ10+MFxuICAgIGlmIHN0YXJ0ZWQ6YWxsb3dlZC5hZGQoJ2hlbHBlcicpXG4gICAgcmF3PWRhdGEuZ2V0KCdvd25lZF9jaGlsZF9leGl0cycpXG4gICAgaWYgaXNpbnN0YW5jZShyYXcsbGlzdCkgYW5kIGxlbihyYXcpPT1sZW4oYWxsb3dlZCkgYW5kIGFsbChpc2luc3RhbmNlKHYsZGljdCkgYW5kIHYuZ2V0KCduYW1lJykgaW4gYWxsb3dlZCBhbmQgdHlwZSh2LmdldCgncGlkJykpIGlzIGludCBhbmQgdlsncGlkJ10+MCBhbmQgdHlwZSh2LmdldCgnZXhpdCcpKSBpcyBpbnQgZm9yIHYgaW4gcmF3KSBhbmQge3ZbJ25hbWUnXSBmb3IgdiBpbiByYXd9PT1hbGxvd2VkIGFuZCBsZW4oe3ZbJ3BpZCddIGZvciB2IGluIHJhd30pPT1sZW4oYWxsb3dlZCkgYW5kIG5leHQodlsncGlkJ10gZm9yIHYgaW4gcmF3IGlmIHZbJ25hbWUnXT09J3NlcnZlcicpPT1jWydzZXJ2ZXJfcGlkJ10gYW5kICgoc3RhcnRlZCBhbmQgbmV4dCh2WydwaWQnXSBmb3IgdiBpbiByYXcgaWYgdlsnbmFtZSddPT0naGVscGVyJyk9PWRhdGFbJ2hlbHBlcl9waWQnXSBhbmQgKGhlbHBlcl9waWQgaXMgTm9uZSBvciBoZWxwZXJfcGlkPT1kYXRhWydoZWxwZXJfcGlkJ10pKSBvciAobm90IHN0YXJ0ZWQgYW5kIGhlbHBlcl9waWQgaXMgTm9uZSBhbmQgZGF0YS5nZXQoJ2hlbHBlcl9pbnZvY2F0aW9ucycpIGlzIE5vbmUgYW5kIGRhdGEuZ2V0KCdoZWxwZXJfcGlkJykgaXMgTm9uZSkpOlxuICAgICAgICBjaGlsZHJlbj1be2s6dltrXSBmb3IgayBpbiAoJ25hbWUnLCdwaWQnLCdleGl0Jyl9IGZvciB2IGluIHJhd11cbiAgICAgICAgaGVscGVyX3BpZD1kYXRhWydoZWxwZXJfcGlkJ10gaWYgc3RhcnRlZCBlbHNlIE5vbmVcbnBpZHM9bGlzdChjWydvd25lZF9waWRzJ10pXG5pZiBoZWxwZXJfcGlkIGlzIG5vdCBOb25lIGFuZCBoZWxwZXJfcGlkIG5vdCBpbiBwaWRzOnBpZHMuYXBwZW5kKGhlbHBlcl9waWQpXG5wPXN1YnByb2Nlc3MucnVuKFsnL2Jpbi9wcycsJy1wJywnLCcuam9pbihzdHIodikgZm9yIHYgaW4gcGlkcyksJy1vJywncGlkPSddLGNhcHR1cmVfb3V0cHV0PVRydWUsdGltZW91dD0zKVxucHNfa25vd249cC5yZXR1cm5jb2RlIGluICgwLDEpIGFuZCBhbGwodi5pc2RpZ2l0KCkgZm9yIHYgaW4gcC5zdGRvdXQuc3BsaXQoKSlcbnByZXNlbnQ9c2V0KGludCh2KSBmb3IgdiBpbiBwLnN0ZG91dC5zcGxpdCgpKSBpZiBwc19rbm93biBlbHNlIHNldCgpXG5wb3J0cz17fVxuZm9yIHBvcnQgaW4gKDkwMDAsMzAwMCk6XG4gICAgcD1zdWJwcm9jZXNzLnJ1bihbJy91c3Ivc2Jpbi9sc29mJywnLW5QJywnLXQnLCctaVRDUDonK3N0cihwb3J0KSwnLXNUQ1A6TElTVEVOJ10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpXG4gICAgcG9ydHNbc3RyKHBvcnQpXT1ub3QgYm9vbChwLnN0ZG91dC5zdHJpcCgpKSBpZiBwLnJldHVybmNvZGUgaW4gKDAsMSkgZWxzZSBOb25lXG5wcmludChqc29uLmR1bXBzKHsnY2xvY2snOnsnbW9ub3RvbmljX25zJzpzdHIodGltZS5tb25vdG9uaWNfbnMoKSksJ3dhbGxfZXBvY2hfbnMnOnN0cih0aW1lLnRpbWVfbnMoKSl9LCdvd25lZF9waWRzJzp7c3RyKHYpOih2IG5vdCBpbiBwcmVzZW50IGlmIHBzX2tub3duIGVsc2UgTm9uZSkgZm9yIHYgaW4gcGlkc30sJ3BvcnRzJzpwb3J0cywnbGFiX2Fic2VudCc6bm90IHBhdGhsaWIuUGF0aChjWydsYWInXSkuZXhpc3RzKCksJ293bmVkX2NoaWxkX2V4aXRzJzpjaGlsZHJlbiwnaGVscGVyX3BpZCc6aGVscGVyX3BpZCwndW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uJzpvdXRlcl9vYnNlcnZhdGlvbiwndW5leHBlY3RlZF9vYnNlcnZhdGlvbl9wcm9qZWN0aW9uJzpwcm9qZWN0aW9ufSkpXG4iOwpjb25zdCBNQVJLRVI9ImltcG9ydCBvcyxwYXRobGliLHN0YXQsc3lzXG5sYWI9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKTtndWFyZD1pbnQoc3lzLmFyZ3ZbMl0pO3NlcnZlcj1pbnQoc3lzLmFyZ3ZbM10pXG5pZiBndWFyZDw9MCBvciBzZXJ2ZXI8PTAgb3IgZ3VhcmQ9PXNlcnZlcjpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKVxuaW5mbz1sYWIubHN0YXQoKVxuaWYgbm90IChzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSk9PTBvNzAwIGFuZCBpbmZvLnN0X3VpZD09b3MuZ2V0dWlkKCkpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5vcy5raWxsKGd1YXJkLDApO29zLmtpbGwoc2VydmVyLDApXG5pZiAobGFiLydzdG9wJykuZXhpc3RzKCkgb3IgKGxhYi8ndWktZmFpbHVyZScpLmV4aXN0cygpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5mZD1vcy5vcGVuKGxhYi8nYnJvd3Nlci1wcmVwYXJlZCcsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbm9zLmNsb3NlKGZkKVxucHJpbnQoJ3tcInByZXBhcmVkX21hcmtlcl93cml0dGVuXCI6dHJ1ZX0nKVxuIjsKY29uc3QgUEVSU0lTVD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzeXNcbnBhdGg9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKVxucmVjb3JkPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMl0pXG4jIENvbnZlcnQgZGVjaW1hbCBzdHJpbmdzIGRpcmVjdGx5IHRvIFB5dGhvbiBpbnRlZ2VycywgYXZvaWRpbmcgSlMgTnVtYmVyIHJvdW5kaW5nLlxuZGVmIGNsb2Nrcyh2YWx1ZSk6XG4gICAgaWYgaXNpbnN0YW5jZSh2YWx1ZSxkaWN0KTpcbiAgICAgICAgZm9yIGssdiBpbiBsaXN0KHZhbHVlLml0ZW1zKCkpOlxuICAgICAgICAgICAgaWYgayBpbiAoJ21vbm90b25pY19ucycsJ3dhbGxfZXBvY2hfbnMnKSBhbmQgaXNpbnN0YW5jZSh2LHN0cik6XG4gICAgICAgICAgICAgICAgYXNzZXJ0IHYuaXNkZWNpbWFsKCkgYW5kIGxlbih2KTw9MTkgYW5kIDA8PWludCh2KTwyKio2M1xuICAgICAgICAgICAgICAgIHZhbHVlW2tdPWludCh2KVxuICAgICAgICAgICAgZWxzZTpjbG9ja3ModilcbiAgICBlbGlmIGlzaW5zdGFuY2UodmFsdWUsbGlzdCk6XG4gICAgICAgIGZvciB2IGluIHZhbHVlOmNsb2Nrcyh2KVxuY2xvY2tzKHJlY29yZClcbmZkPW9zLm9wZW4ocGF0aCxvcy5PX1dST05MWXxvcy5PX0NSRUFUfG9zLk9fRVhDTHxvcy5PX05PRk9MTE9XLDBvNjAwKVxud2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgZi53cml0ZShqc29uLmR1bXBzKHJlY29yZCxzb3J0X2tleXM9VHJ1ZSxpbmRlbnQ9MikrJ1xcbicpO2YuZmx1c2goKTtvcy5mc3luYyhmLmZpbGVubygpKVxucHJpbnQoanNvbi5kdW1wcyh7J3dyaXR0ZW5fZXhjbHVzaXZlJzpUcnVlfSkpXG4iOwpjb25zdCBvd249bG9hZCgiZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIik7CmNvbnN0IE9CU0tFWT0iZDAxX2ltbWVkaWF0ZV9vYnNlcnZhdGlvbl9yZWNvcmQiOwppZiAoIW93biB8fCBvd24ucnVudGltZV9yZWxlYXNlIT09dHJ1ZSB8fCBvd24uZHJpdmVyX293bmVkIT09dHJ1ZSB8fAogICAgb3duLmV4YWN0X2JvdW5kIT09dHJ1ZSB8fCBvd24uc2luZ2xlX2NvbnRyb2xsZXIhPT10cnVlIHx8CiAgICAhWyJwcmVwYXJlZF9lbnRyeSIsImJyb3dzZXJfZGVjaXNpb24iXS5pbmNsdWRlcyhvd24ucGhhc2UpIHx8CiAgICAob3duLnBoYXNlPT09InByZXBhcmVkX2VudHJ5IiYmKG93bi5wcmVwYXJlZF9nYXRlIT09dHJ1ZXx8b3duLmhlbHBlcl9waWQhPT1udWxsfHwKICAgICAgb3duLmZpeHR1cmVfcmVhZHk9PT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mPT09dHJ1ZSkpIHx8CiAgICAob3duLnBoYXNlPT09ImJyb3dzZXJfZGVjaXNpb24iJiYob3duLmZpeHR1cmVfcmVhZHkhPT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mIT09dHJ1ZSkpIHx8CiAgICBvd24uZnJlc2hfcGF0aHNfcHJlZmxpZ2h0IT09dHJ1ZSB8fAogICAgIU51bWJlci5pc1NhZmVJbnRlZ2VyKG93bi5zdGFydF9lcG9jaF9tcykgfHwgdHlwZW9mIG93bi53b3Jrc3BhY2UhPT0ic3RyaW5nIiB8fAogICAgbmV3IFNldChbb3duLmd1YXJkX3BpZCxvd24uc2VydmVyX3BpZCxvd24uYnJvd3Nlcl9waWQsCiAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSkuc2l6ZSE9PShvd24uaGVscGVyX3BpZD09PW51bGw/Mzo0KSB8fAogICAgIVtvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCxvd24uZXhlY19zZXNzaW9uLAogICAgICAgLi4uKG93bi5oZWxwZXJfcGlkPT09bnVsbD9bXTpbb3duLmhlbHBlcl9waWRdKV0KICAgICAgIC5ldmVyeSh2PT5OdW1iZXIuaXNTYWZlSW50ZWdlcih2KSYmdj4wKSB8fAogICAgIVtvd24uc2Vzc2lvbixvd24udGFyZ2V0X2lkLG93bi50YWJfaWQsb3duLmxhYixvd24ub3V0ZXJfb3V0LAogICAgICAgb3duLmV2ZW50X291dCxvd24uY2xlYW51cF9vdXRdLmV2ZXJ5KHY9PnR5cGVvZiB2PT09InN0cmluZyImJnYubGVuZ3RoPjApKSB7CiAgdGV4dCh7cHJvcG9zYWxfcmVmdXNlZDoiZnJlc2hfb3duZWRfY29udGV4dF9yZXF1aXJlZCJ9KTtleGl0KCk7Cn0KY29uc3QgcHJpb3I9bG9hZChPQlNLRVkpOwppZihvd24uY2xvc2VkPT09dHJ1ZXx8cHJpb3I/LmNsZWFudXBfc3RhcnRlZD09PXRydWUpewogIHRleHQoe3Byb3Bvc2FsX3JlZnVzZWQ6ImZpeHR1cmVfYWxyZWFkeV9zdG9wcGVkIixyZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpmYWxzZX0pO2V4aXQoKTsKfQpjb25zdCByZWNvcmQ9cHJpb3I/P3sKICBzY2hlbWE6InJpYXV0aC5kMDEtaW1tZWRpYXRlLW9ic2VydmF0aW9uLWNsZWFudXAvdjEiLAogIGZpcnN0X2ZhaWx1cmU6bnVsbCxmaXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOm51bGwsCiAgZmlyc3RfZXZlbnRfcHJvdmVuOmZhbHNlLHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogIHVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbjpudWxsLGRpYWdub3N0aWNfZXJyb3JzOltdLGNsZWFudXBfc3RhcnRlZDpmYWxzZSwKICBmaXJzdF9jbG9jazpudWxsLG9ic2VydmF0aW9uX3JlY2VpcHRzOltdLGNvbnRyb2xsZXJfb2JzZXJ2YXRpb25zOltdLGxvY2FsX3Rvb2xfcmVjZWlwdHM6W10sYWN0aW9uczpbXSxjbGVhbnVwX2Vycm9yczpbXSwKICBmaW5hbF9hYnNlbmNlOm51bGwsb3duZWRfY2hpbGRfZXhpdHM6bnVsbCxjb250cm9sbGVyX2V4aXQ6bnVsbCwKICBmaXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zOm51bGwsbGF0Y2hfdG9fYWJzZW5jZV9tczpudWxsCn07CmNvbnN0IHNxPXM9PiInIitTdHJpbmcocykucmVwbGFjZSgvJy9nLCInXFwnJyIpKyInIjsKY29uc3QgcmV0YWluPSgpPT5zdG9yZShPQlNLRVkscmVjb3JkKTsKY29uc3QgY2xlYW51cEVycm9yPWxhYmVsPT57CiAgaWYoIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5pbmNsdWRlcyhsYWJlbCkpcmVjb3JkLmNsZWFudXBfZXJyb3JzLnB1c2gobGFiZWwpOwogIHJldGFpbigpOwp9Owpjb25zdCBsb2NhbD1hc3luYyhzb3VyY2UsYXJnKT0+ewogIGNvbnN0IGNtZD0icHl0aG9uMyAtYyAiK3NxKHNvdXJjZSkrKGFyZz09PXVuZGVmaW5lZD8iIjoiICIrc3EoSlNPTi5zdHJpbmdpZnkoYXJnKSkpOwogIGNvbnN0IHI9YXdhaXQgdG9vbHMuZXhlY19jb21tYW5kKHtjbWQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgeWllbGRfdGltZV9tczoxMDAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7CiAgICBsYWJlbDpzb3VyY2U9PT1DTE9DSz8iY2xvY2siOnNvdXJjZT09PVNUT1A/InN0b3AiOnNvdXJjZT09PVJFQURCQUNLPyJyZWFkYmFjayI6InVua25vd24iLAogICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGwKICB9KTtyZXRhaW4oKTsgLy8gTnVtZXJpYyBjb2xsZWN0b3IgcmVjZWlwdCBCRUZPUkUgY29tcGFyaXNvbnMuCiAgaWYoci5zZXNzaW9uX2lkIT09dW5kZWZpbmVkKSB7CiAgICBjbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTt0aHJvdyBuZXcgRXJyb3IoImxvY2FsX3BlbmRpbmciKTsKICB9CiAgaWYoci5leGl0X2NvZGUhPT0wKXRocm93IG5ldyBFcnJvcigibG9jYWxfZmFpbGVkIik7CiAgcmV0dXJuIEpTT04ucGFyc2Uoci5vdXRwdXQpOwp9OwpmdW5jdGlvbiBmaW5pdGVPYnNlcnZhdGlvbih2YWx1ZSkgewogIGNvbnN0IGV4YWN0PSh2LGtleXMpPT52IT09bnVsbCYmdHlwZW9mIHY9PT0ib2JqZWN0IiYmIUFycmF5LmlzQXJyYXkodikmJgogICAgT2JqZWN0LmtleXModikubGVuZ3RoPT09a2V5cy5sZW5ndGgmJmtleXMuZXZlcnkoaz0+T2JqZWN0Lmhhc093bih2LGspKTsKICBpZighZXhhY3QodmFsdWUsWyJvYnNlcnZlZCIsImRpYWdub3N0aWMiXSl8fHR5cGVvZiB2YWx1ZS5vYnNlcnZlZCE9PSJib29sZWFuIilyZXR1cm4gbnVsbDsKICBjb25zdCBkPXZhbHVlLmRpYWdub3N0aWM7CiAgaWYoZD09PW51bGwpcmV0dXJuIHtvYnNlcnZlZDp2YWx1ZS5vYnNlcnZlZCxkaWFnbm9zdGljOm51bGx9OwogIGlmKCF2YWx1ZS5vYnNlcnZlZHx8IWV4YWN0KGQsWyJzaXRlIiwiZXhjZXB0aW9uX2NsYXNzIiwib3duX2Z1bmN0aW9uIiwib3duX2xpbmUiXSl8fAogICAgICFbImhhbmRsZXIiLCJzZXJ2ZXIiLCJtYWluIl0uaW5jbHVkZXMoZC5zaXRlKXx8CiAgICAgIVsiQXR0cmlidXRlRXJyb3IiLCJUeXBlRXJyb3IiLCJWYWx1ZUVycm9yIiwiS2V5RXJyb3IiLCJPU0Vycm9yIiwiQnJva2VuUGlwZUVycm9yIiwKICAgICAgICJDb25uZWN0aW9uUmVzZXRFcnJvciIsIlRpbWVvdXRFcnJvciIsIm90aGVyIl0uaW5jbHVkZXMoZC5leGNlcHRpb25fY2xhc3MpKXJldHVybiBudWxsOwogIGNvbnN0IGZ1bmN0aW9ucz1bIkhlYWRlclJlYWRlci5yZWFkbGluZSIsIkRlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0IiwiRGVtby5iZWdpbiIsIkRlbW8uY2FsbGJhY2siLAogICAgIkRlbW8uaW52b2tlIiwiSGFuZGxlci5oYW5kbGVfb25lX3JlcXVlc3QiLCJIYW5kbGVyLnNlbmRfZXJyb3IiLCJIYW5kbGVyLmdldCIsIkhhbmRsZXIucmVwbHkiLCJtYWluIl07CiAgaWYoISgoZC5vd25fZnVuY3Rpb249PT1udWxsJiZkLm93bl9saW5lPT09bnVsbCl8fAogICAgICAgKGZ1bmN0aW9ucy5pbmNsdWRlcyhkLm93bl9mdW5jdGlvbikmJk51bWJlci5pc1NhZmVJbnRlZ2VyKGQub3duX2xpbmUpJiYKICAgICAgICBkLm93bl9saW5lPj0xJiZkLm93bl9saW5lPD0xMDI0KSkpcmV0dXJuIG51bGw7CiAgcmV0dXJuIHtvYnNlcnZlZDp0cnVlLGRpYWdub3N0aWM6e3NpdGU6ZC5zaXRlLGV4Y2VwdGlvbl9jbGFzczpkLmV4Y2VwdGlvbl9jbGFzcywKICAgIG93bl9mdW5jdGlvbjpkLm93bl9mdW5jdGlvbixvd25fbGluZTpkLm93bl9saW5lfX07Cn0KZnVuY3Rpb24gZGlhZ25vc3RpYyh2YWx1ZSxwcm9qZWN0aW9uKSB7CiAgY29uc3QgcHJvamVjdGVkPWZpbml0ZU9ic2VydmF0aW9uKHZhbHVlKTsKICBpZihwcm9qZWN0aW9uPT09ImludmFsaWQifHwhWyJ2YWxpZCIsInVuYXZhaWxhYmxlIl0uaW5jbHVkZXMocHJvamVjdGlvbil8fAogICAgIChwcm9qZWN0aW9uPT09InZhbGlkIiYmcHJvamVjdGVkPT09bnVsbCkpIHsKICAgIGlmKCFyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMuaW5jbHVkZXMoIm9ic2VydmVyX3Byb2plY3Rpb25faW52YWxpZCIpKQogICAgICByZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMucHVzaCgib2JzZXJ2ZXJfcHJvamVjdGlvbl9pbnZhbGlkIik7CiAgfQogIGlmKHByb2plY3RlZCE9PW51bGwmJnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb249PT1udWxsKQogICAgcmVjb3JkLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbj1wcm9qZWN0ZWQ7CiAgcmV0YWluKCk7IC8vIE5vIHJhdyBkaWFnbm9zdGljIGZhbGxiYWNrIGFuZCBubyBmaXJzdC1mYWlsdXJlIG9yIG91dGNvbWUgbXV0YXRpb24uCn0KY29uc3QgbnM9Yz0+QmlnSW50KGMubW9ub3RvbmljX25zKTsKY29uc3QgZ2V0Q2xvY2s9KCk9PmxvY2FsKENMT0NLKTsKbGV0IGNvbnRyb2xsZXJKb2luZWQ9ZmFsc2U7CmxldCBjb250cm9sbGVyUG9sbEF2YWlsYWJsZT10cnVlOwpsZXQgY29udHJvbGxlckJ1ZmZlcj0iIjsKbGV0IGNsZWFudXBTdGFydGVkPWZhbHNlOwpsZXQgbGFzdFNuYXBzaG90RmxhZ3M9bnVsbDsKZnVuY3Rpb24gbGF0Y2gobGFiZWwscmVjZWl2ZWRXYWxsKSB7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICByZWNvcmQuZmlyc3RfZmFpbHVyZT1sYWJlbDsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPXJlY2VpdmVkV2FsbDsKICAgIHJldGFpbigpOyAvLyBTeW5jaHJvbm91cyBmaXJzdC1mYWlsdXJlL3dhbGwgbGF0Y2ggQkVGT1JFIGFueSBuZXcgYXdhaXQvb3V0cHV0LgogIH0KfQpmdW5jdGlvbiBvYnNlcnZlQ29udHJvbGxlcihyLHJlY2VpdmVkV2FsbCkgewogIGNvbnN0IG9ic2VydmF0aW9uPXtyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbCwKICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgIGhlbHBlcl9jb21wbGV0ZWQ6ZmFsc2UsaGVscGVyX2V4aXQ6bnVsbCxmaXh0dXJlX2ZpbmlzaGVkOmZhbHNlfTsKICAvLyBDb21wbGV0ZSBudW1lcmljIHJlc3VsdCBwcm9qZWN0aW9uIGlzIHJldGFpbmVkIEJFRk9SRSBjb21wYXJpc29ucy4KICByZWNvcmQuY29udHJvbGxlcl9vYnNlcnZhdGlvbnMucHVzaChvYnNlcnZhdGlvbik7cmV0YWluKCk7CiAgaWYodHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciIpIHsKICAgIGNvbnRyb2xsZXJKb2luZWQ9dHJ1ZTtyZWNvcmQuY29udHJvbGxlcl9leGl0PXIuZXhpdF9jb2RlO3JldGFpbigpOwogIH0KICBjb250cm9sbGVyQnVmZmVyKz10eXBlb2Ygci5vdXRwdXQ9PT0ic3RyaW5nIj9yLm91dHB1dDoiIjsKICBpZihjb250cm9sbGVyQnVmZmVyLmxlbmd0aD4xNjM4NCkgewogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250cm9sbGVyQnVmZmVyPSIiO3JldHVybjsKICB9CiAgbGV0IGN1dDsKICB3aGlsZSgoY3V0PWNvbnRyb2xsZXJCdWZmZXIuaW5kZXhPZigiXG4iKSk+PTApIHsKICAgIGNvbnN0IGxpbmU9Y29udHJvbGxlckJ1ZmZlci5zbGljZSgwLGN1dCk7Y29udHJvbGxlckJ1ZmZlcj1jb250cm9sbGVyQnVmZmVyLnNsaWNlKGN1dCsxKTsKICAgIGlmKCFsaW5lLnRyaW0oKSljb250aW51ZTsKICAgIGxldCBldmVudDsKICAgIHRyeXtldmVudD1KU09OLnBhcnNlKGxpbmUpO31jYXRjaHsKICAgICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250aW51ZTsKICAgIH0KICAgIGlmKE9iamVjdC5oYXNPd24oZXZlbnQsInVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiIpKQogICAgICBkaWFnbm9zdGljKGV2ZW50LnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbixldmVudC51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoZXZlbnQuZml4dHVyZV9yZWFkeT09PXRydWUpIHsKICAgICAgaWYoZXZlbnQuZ3VhcmRfcGlkIT09b3duLmd1YXJkX3BpZHx8ZXZlbnQuc2VydmVyX3BpZCE9PW93bi5zZXJ2ZXJfcGlkfHwKICAgICAgICAgZXZlbnQubGFiIT09b3duLmxhYnx8IU51bWJlci5pc1NhZmVJbnRlZ2VyKGV2ZW50LmhlbHBlcl9waWQpfHxldmVudC5oZWxwZXJfcGlkPD0wfHwKICAgICAgICAgW293bi5ndWFyZF9waWQsb3duLnNlcnZlcl9waWQsb3duLmJyb3dzZXJfcGlkXS5pbmNsdWRlcyhldmVudC5oZWxwZXJfcGlkKXx8CiAgICAgICAgIChvd24uaGVscGVyX3BpZCE9PW51bGwmJm93bi5oZWxwZXJfcGlkIT09ZXZlbnQuaGVscGVyX3BpZCkpCiAgICAgICAgbGF0Y2goImZpeHR1cmVfcmVhZHlfdW5jb25maXJtZWQiLHJlY2VpdmVkV2FsbCk7CiAgICAgIGVsc2UgewogICAgICAgIG93bi5oZWxwZXJfcGlkPWV2ZW50LmhlbHBlcl9waWQ7b3duLmZpeHR1cmVfcmVhZHk9dHJ1ZTtvd24ubGlzdGVuZXJfcGlkX3Byb29mPXRydWU7CiAgICAgICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICAgICAgfQogICAgfQogICAgaWYoZXZlbnQuaGVscGVyX2NvbXBsZXRlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uaGVscGVyX2NvbXBsZXRlZD10cnVlOwogICAgICBvYnNlcnZhdGlvbi5oZWxwZXJfZXhpdD10eXBlb2YgZXZlbnQuZXhpdD09PSJudW1iZXIiP2V2ZW50LmV4aXQ6bnVsbDsKICAgICAgcmV0YWluKCk7CiAgICAgIGlmKGV2ZW50LmV4aXQhPT0wKWxhdGNoKCJoZWxwZXJfZmFpbGVkIixyZWNlaXZlZFdhbGwpOwogICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSYmCiAgICAgICAgICAgICAgIShvd24ucGhhc2U9PT0iYnJvd3Nlcl9kZWNpc2lvbiImJgogICAgICAgICAgICAgICAgb3duLm5leHRfZGVjaXNpb24/LmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIikpCiAgICAgICAgbGF0Y2goImhlbHBlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50IixyZWNlaXZlZFdhbGwpOwogICAgfQogICAgaWYoZXZlbnQuZml4dHVyZV9maW5pc2hlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uZml4dHVyZV9maW5pc2hlZD10cnVlO3JldGFpbigpOwogICAgICBpZighY2xlYW51cFN0YXJ0ZWQpbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKICAgIH0KICB9CiAgaWYoY29udHJvbGxlckpvaW5lZCYmIWNsZWFudXBTdGFydGVkKQogICAgbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKfQphc3luYyBmdW5jdGlvbiBwb2xsQ29udHJvbGxlcigpIHsKICBpZihjb250cm9sbGVySm9pbmVkfHwhY29udHJvbGxlclBvbGxBdmFpbGFibGUpcmV0dXJuOwogIHRyeSB7CiAgICBjb25zdCByPWF3YWl0IHRvb2xzLndyaXRlX3N0ZGluKHtzZXNzaW9uX2lkOm93bi5leGVjX3Nlc3Npb24sCiAgICAgIGNoYXJzOiIiLHlpZWxkX3RpbWVfbXM6NTAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgICBjb25zdCByZWNlaXZlZFdhbGw9RGF0ZS5ub3coKTsKICAgIG9ic2VydmVDb250cm9sbGVyKHIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHsKICAgIGNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlPWZhbHNlOwogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25fdW5hdmFpbGFibGUiLERhdGUubm93KCkpOwogICAgaWYoY2xlYW51cFN0YXJ0ZWQpY2xlYW51cEVycm9yKCJjb250cm9sbGVyX2pvaW5fdW5jb25maXJtZWQiKTsKICB9Cn0KYXN5bmMgZnVuY3Rpb24gdGltZWREcml2ZXIobGFiZWwsYWxsb2NhdGlvbixvcGVyYXRpb24scHJvamVjdCkgewogIGxldCBzdGFydD1udWxsLGVuZD1udWxsLHJlc3VsdD1udWxsLHN0YXRlPSJ1bmtub3duIjsKICB0cnl7c3RhcnQ9YXdhaXQgZ2V0Q2xvY2soKTt9Y2F0Y2h7Y2xlYW51cEVycm9yKCJjbG9ja191bmF2YWlsYWJsZSIpO30KICBjb25zdCBhY3Rpb249e2xhYmVsLHN0YXJ0X2Nsb2NrOnN0YXJ0LGVuZF9jbG9jazpudWxsLGFsbG93ZWRfbXM6YWxsb2NhdGlvbiwKICAgIHJlc3VsdF9zdGF0ZToidW5rbm93biIsZWxhcHNlZF9tczpudWxsLG92ZXJfYnVkZ2V0Om51bGx9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsgLy8gU3RhcnQgcmV0YWluZWQgQkVGT1JFIGVudGVyaW5nIERyaXZlciBjYWxsLgogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7CiAgICBzdGF0ZT1yZXN1bHQ/LmlzRXJyb3I9PT10cnVlPyJyZWZ1c2VkIjoicmV0dXJuZWQiOwogIH0gY2F0Y2gge3N0YXRlPSJleGNlcHRpb24iO30KICB0cnl7ZW5kPWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgYWN0aW9uLmVuZF9jbG9jaz1lbmQ7YWN0aW9uLnJlc3VsdF9zdGF0ZT1zdGF0ZTsKICByZXRhaW4oKTsgLy8gRW5kL3Jlc3VsdCByZXRhaW5lZCBCRUZPUkUgYnVkZ2V0IGNvbXBhcmlzb24uCiAgaWYoc3RhcnQmJmVuZCkgewogICAgY29uc3QgZD1ucyhlbmQpLW5zKHN0YXJ0KTsKICAgIGlmKGQ+PTBuKSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcihkLzEwMDAwMDBuKTsKICAgICAgYWN0aW9uLm92ZXJfYnVkZ2V0PWFjdGlvbi5lbGFwc2VkX21zPmFsbG9jYXRpb247CiAgICB9IGVsc2UgY2xlYW51cEVycm9yKCJjbG9ja19pbnZhbGlkIik7CiAgfQogIGlmKGFjdGlvbi5vdmVyX2J1ZGdldCljbGVhbnVwRXJyb3IoImRyaXZlcl9vcGVyYXRpb25fb3Zlcl9idWRnZXQiKTsKICBpZihzdGF0ZSE9PSJyZXR1cm5lZCIpY2xlYW51cEVycm9yKCJkcml2ZXJfb3BlcmF0aW9uX3VuY29uZmlybWVkIik7CiAgaWYocHJvamVjdClwcm9qZWN0KHJlc3VsdCxzdGF0ZSk7CiAgcmV0YWluKCk7Cn0KYXN5bmMgZnVuY3Rpb24gY2xlYW51cCgpIHsKICBzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsgLy8gQ2xlYXIgYmVmb3JlIGFueSBjbGVhbnVwIGF3YWl0LgogIGlmKGNsZWFudXBTdGFydGVkKXJldHVybjsKICBjbGVhbnVwU3RhcnRlZD10cnVlO3JlY29yZC5jbGVhbnVwX3N0YXJ0ZWQ9dHJ1ZTtyZXRhaW4oKTsKICB0cnkgewogICAgY29uc3Qgcj1hd2FpdCBsb2NhbChTVE9QLHsKICAgICAgbGFiOm93bi5sYWIsZXZlbnRfb3V0Om93bi5ldmVudF9vdXQsCiAgICAgIGZpcnN0X2ZhaWx1cmU6cmVjb3JkLmZpcnN0X2ZhaWx1cmUsCiAgICAgIGZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXM6cmVjb3JkLmZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXMKICAgIH0pOwogICAgcmVjb3JkLmZpcnN0X2Nsb2NrPXIuY2xvY2s7cmVjb3JkLnN0b3Bfc3RhdGU9ci5tYXJrZXJfc3RhdGU7cmV0YWluKCk7CiAgICBpZihyLmV2ZW50X3N0YXRlIT09IndyaXR0ZW4iKWNsZWFudXBFcnJvcigib2JzZXJ2YXRpb25fbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICAgIGlmKHIubWFya2VyX3N0YXRlPT09InN0b3BfdW5jb25maXJtZWQiKWNsZWFudXBFcnJvcigic3RvcF91bmNvbmZpcm1lZCIpOwogICAgaWYoci5tYXJrZXJfc3RhdGU9PT0ib3duZXJzaGlwX3Vua25vd24iKWNsZWFudXBFcnJvcigib3duZXJzaGlwX3Vua25vd24iKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoInN0b3Bfb3JfY2xvY2tfcmVjb3JkX3VuYXZhaWxhYmxlIik7fQogIC8vIE5vIG91dHN0YW5kaW5nIERyaXZlciBjYWxsIGV4aXN0cyBoZXJlOiBhbGwgcGFnZSBjYWxscyBhYm92ZSB3ZXJlIGF3YWl0ZWQuCiAgLy8gT3JpZ2luYWwgc3RvcCBwcm90b2NvbCBhbHJlYWR5IGNhdXNlcyB0aGUgY29udHJvbGxlcidzIG93bmVkLWNoaWxkIGZpbmFsbHkuCiAgYXdhaXQgdGltZWREcml2ZXIoImtpbGxfYXBwIiwzMDAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2tpbGxfYXBwKHtwaWQ6b3duLmJyb3dzZXJfcGlkfSkpOwogIGF3YWl0IHRpbWVkRHJpdmVyKCJlbmRfc2Vzc2lvbiIsMTUwMDAsCiAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19lbmRfc2Vzc2lvbih7c2Vzc2lvbjpvd24uc2Vzc2lvbn0pLChyLHN0YXRlKT0+ewogICAgICByZWNvcmQuc2Vzc2lvbl9lbmRlZD1zdGF0ZT09PSJyZXR1cm5lZCImJgogICAgICAgIHI/LnN0cnVjdHVyZWRDb250ZW50Py5hY3RpdmU9PT1mYWxzZSYmci5zdHJ1Y3R1cmVkQ29udGVudC5zZXNzaW9uPT09b3duLnNlc3Npb247CiAgICAgIHJldGFpbigpOwogICAgfSk7CiAgYXdhaXQgdGltZWREcml2ZXIoImxpc3Rfd2luZG93cyIsNTAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2xpc3Rfd2luZG93cyh7cGlkOm93bi5icm93c2VyX3BpZH0pLChyLHN0YXRlKT0+ewogICAgICBjb25zdCB3aW5kb3dzPXI/LnN0cnVjdHVyZWRDb250ZW50Py53aW5kb3dzOwogICAgICByZWNvcmQud2luZG93X2NvdW50PXN0YXRlPT09InJldHVybmVkIiYmQXJyYXkuaXNBcnJheSh3aW5kb3dzKT93aW5kb3dzLmxlbmd0aDpudWxsOwogICAgICByZXRhaW4oKTsKICAgIH0pOwogIGxldCByZWFkU3RhcnQ9bnVsbDsKICB0cnl7cmVhZFN0YXJ0PWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgY29uc3QgYWN0aW9uPXtsYWJlbDoib3duZWRfam9pbl9yZWFkYmFjayIsc3RhcnRfY2xvY2s6cmVhZFN0YXJ0LGVuZF9jbG9jazpudWxsLAogICAgYWxsb3dlZF9tczpudWxsLGVsYXBzZWRfbXM6bnVsbCxvdmVyX2J1ZGdldDpudWxsLHJlc3VsdF9zdGF0ZToidW5rbm93biJ9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsKICBpZihyZWFkU3RhcnQmJnJlY29yZC5maXJzdF9jbG9jaykgewogICAgYWN0aW9uLmFsbG93ZWRfbXM9TWF0aC5tYXgoMCw2MDAwMC1OdW1iZXIoKG5zKHJlYWRTdGFydCktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pKTsKICAgIHJldGFpbigpOwogIH0KICAvLyBDb250aW51ZSBlc3NlbnRpYWwgam9pbiBhZnRlcjYwcyBpZiBsYXRlOyBuZXZlciBjbGFpbSB0aGF0IGRlYWRsaW5lIGVuZm9yY2VkLgogIC8vIERvIG5vdCBwb2xsIGFub3RoZXIgZXhlYyBzZXNzaW9uIG9yIHNlbmQgYSBtYW51YWwgcHJvY2VzcyBzaWduYWwuCiAgd2hpbGUoIWNvbnRyb2xsZXJKb2luZWQmJmNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlJiZEYXRlLm5vdygpPG93bi5zdGFydF9lcG9jaF9tcys5MDAwMDApIHsKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spIHsKICAgICAgdHJ5IHsKICAgICAgICBjb25zdCBjPWF3YWl0IGdldENsb2NrKCk7cmVjb3JkLmxhc3Rfam9pbl9jbG9jaz1jO3JldGFpbigpOwogICAgICAgIGlmKG5zKGMpLW5zKHJlY29yZC5maXJzdF9jbG9jayk+NjAwMDAwMDAwMDBuKQogICAgICAgICAgY2xlYW51cEVycm9yKCJjbGVhbnVwX2J1ZGdldF9leGNlZWRlZCIpOwogICAgICB9IGNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgICB9CiAgfQogIGlmKCFjb250cm9sbGVySm9pbmVkKWNsZWFudXBFcnJvcigiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIik7CiAgdHJ5IHsKICAgIGNvbnN0IHI9YXdhaXQgbG9jYWwoUkVBREJBQ0ssewogICAgICBvd25lZF9waWRzOltvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCwKICAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSwKICAgICAgbGFiOm93bi5sYWIsb3V0ZXJfb3V0Om93bi5vdXRlcl9vdXQsaGVscGVyX3BpZDpvd24uaGVscGVyX3BpZCxzZXJ2ZXJfcGlkOm93bi5zZXJ2ZXJfcGlkCiAgICB9KTsKICAgIGFjdGlvbi5lbmRfY2xvY2s9ci5jbG9jazthY3Rpb24ucmVzdWx0X3N0YXRlPSJyZXR1cm5lZCI7CiAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHM9ci5vd25lZF9jaGlsZF9leGl0czsKICAgIGRpYWdub3N0aWMoci51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sci51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoTnVtYmVyLmlzU2FmZUludGVnZXIoci5oZWxwZXJfcGlkKSYmci5oZWxwZXJfcGlkPjApb3duLmhlbHBlcl9waWQ9ci5oZWxwZXJfcGlkOwogICAgcmVjb3JkLmZpbmFsX2Fic2VuY2U9e293bmVkX3BpZHM6ci5vd25lZF9waWRzLHBvcnRzOnIucG9ydHMsCiAgICAgIGxhYjpyLmxhYl9hYnNlbnQsd2luZG93X2NvdW50OnJlY29yZC53aW5kb3dfY291bnQ/P251bGwsCiAgICAgIHNlc3Npb25fZW5kZWQ6cmVjb3JkLnNlc3Npb25fZW5kZWQ9PT10cnVlfTsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zPXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPT09bnVsbD9udWxsOgogICAgICBEYXRlLm5vdygpLXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOwogICAgcmV0YWluKCk7IC8vIEZ1bGwgZml4ZWQgcmVhZGJhY2svZXhpdHMvY2xvY2sgQkVGT1JFIGNvbXBhcmlzb25zLgogICAgaWYocmVhZFN0YXJ0KSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVhZFN0YXJ0KSkvMTAwMDAwMG4pOwogICAgICBhY3Rpb24ub3Zlcl9idWRnZXQ9YWN0aW9uLmFsbG93ZWRfbXM9PT1udWxsP251bGw6YWN0aW9uLmVsYXBzZWRfbXM+YWN0aW9uLmFsbG93ZWRfbXM7CiAgICB9CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spCiAgICAgIHJlY29yZC5sYXRjaF90b19hYnNlbmNlX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pOwogICAgaWYoYWN0aW9uLm92ZXJfYnVkZ2V0KWNsZWFudXBFcnJvcigiY2xlYW51cF9idWRnZXRfZXhjZWVkZWQiKTsKICAgIGNvbnN0IGE9cmVjb3JkLmZpbmFsX2Fic2VuY2U7CiAgICBpZighY29udHJvbGxlckpvaW5lZHx8IUFycmF5LmlzQXJyYXkocmVjb3JkLm93bmVkX2NoaWxkX2V4aXRzKXx8CiAgICAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHMubGVuZ3RoIT09KG93bi5oZWxwZXJfcGlkPT09bnVsbD82OjcpKQogICAgICBjbGVhbnVwRXJyb3IoImNoaWxkX3JlYXBfaW5jb21wbGV0ZSIpOwogICAgaWYoIU9iamVjdC52YWx1ZXMoYS5vd25lZF9waWRzKS5ldmVyeSh2PT52PT09dHJ1ZSl8fAogICAgICAgIU9iamVjdC52YWx1ZXMoYS5wb3J0cykuZXZlcnkodj0+dj09PXRydWUpfHxhLmxhYiE9PXRydWV8fAogICAgICAgYS53aW5kb3dfY291bnQhPT0wfHxhLnNlc3Npb25fZW5kZWQhPT10cnVlKQogICAgICBjbGVhbnVwRXJyb3IoIm93bmVkX3Jlc291cmNlX2Fic2VuY2VfdW5wcm92ZW4iKTsKICB9IGNhdGNoIHthY3Rpb24ucmVzdWx0X3N0YXRlPSJleGNlcHRpb24iO2NsZWFudXBFcnJvcigib3duZWRfcmVhZGJhY2tfdW5hdmFpbGFibGUiKTt9CiAgY2xlYW51cEVycm9yKCJmaXJzdF9ldmVudF91bnByb3ZlbiIpOwogIHJlY29yZC53aG9sZV9jbGVhbnVwX3dpdGhpbjYwX3Byb3Zlbj1mYWxzZTtyZXRhaW4oKTsKICAvLyBFeGNsdXNpdmUgbmV3IGZpbGUgb25seTsgZXhpc3Rpbmcgb3V0ZXIvaGVscGVyL3Byb3ZpZGVyIGV2aWRlbmNlIHVudG91Y2hlZC4KICB0cnkgewogICAgY29uc3QgY21kPSJweXRob24zIC1jICIrc3EoUEVSU0lTVCkrIiAiK3NxKG93bi5jbGVhbnVwX291dCkrIiAiK3NxKEpTT04uc3RyaW5naWZ5KHJlY29yZCkpOwogICAgY29uc3Qgcj1hd2FpdCB0b29scy5leGVjX2NvbW1hbmQoe2NtZCx3b3JrZGlyOm93bi53b3Jrc3BhY2UsCiAgICAgIHlpZWxkX3RpbWVfbXM6MTAwMDAsbWF4X291dHB1dF90b2tlbnM6NTAwfSk7CiAgICByZWNvcmQubG9jYWxfdG9vbF9yZWNlaXB0cy5wdXNoKHtsYWJlbDoicGVyc2lzdCIsCiAgICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGx9KTtyZXRhaW4oKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZHx8ci5leGl0X2NvZGUhPT0wKQogICAgICBjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTt9CiAgY29udHJvbGxlckJ1ZmZlcj0iIjtvd24uY2xvc2VkPXRydWU7c3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKfQphc3luYyBmdW5jdGlvbiBjaGVja2VkKGxhYmVsLG9wZXJhdGlvbixwcmVkaWNhdGUpIHsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpcmV0dXJuIG51bGw7CiAgaWYoRGF0ZS5ub3coKT49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkgewogICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuIG51bGw7CiAgfQogIGxldCByZXN1bHQscmVjZWl2ZWRXYWxsOwogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7cmVjZWl2ZWRXYWxsPURhdGUubm93KCk7CiAgICByZWNvcmQub2JzZXJ2YXRpb25fcmVjZWlwdHMucHVzaCh7a2luZDpsYWJlbCxyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbH0pO3JldGFpbigpOwogICAgaWYocmVzdWx0Py5pc0Vycm9yPT09dHJ1ZSlsYXRjaCgiYnJvd3Nlcl90b29sX3JlZnVzZWQiLHJlY2VpdmVkV2FsbCk7CiAgICBlbHNlIGlmKCFwcmVkaWNhdGUocmVzdWx0KSlsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHtsYXRjaCgiYnJvd3Nlcl90b29sX29yX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7fQogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVhbnVwKCk7CiAgcmV0dXJuIHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbD9yZXN1bHQ6bnVsbDsKfQpjb25zdCBzbmFwc2hvdEFyZ3M9e3Nlc3Npb246b3duLnNlc3Npb24sdGFyZ2V0X2lkOm93bi50YXJnZXRfaWQsdGFiX2lkOm93bi50YWJfaWQsCiAgc25hcHNob3RfZm9ybWF0OiJzZW1hbnRpY192MiIsaW5jbHVkZV9zY3JlZW5zaG90OmZhbHNlfTsKZnVuY3Rpb24gcGFnZShyZXN1bHQsa2luZCkgewogIGNvbnN0IHM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudCxub2Rlcz1BcnJheS5pc0FycmF5KHM/LmNvbnRlbnRfcmVmcyk/cy5jb250ZW50X3JlZnM6W107CiAgY29uc3QgZmxhZ3M9ewogICAgc3RhdHVzX29rOnJlc3VsdD8uaXNFcnJvciE9PXRydWUmJnM/LnN0YXR1cz09PSJvayIsCiAgICBjb21wbGV0ZTpzPy5zbmFwc2hvdD8uY29tcGxldGU9PT10cnVlLAogICAgaGVhZGluZ19tYXRjaDpub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09CiAgICAgIChraW5kPT09InByb3RlY3RlZF9hZnRlciI/IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiOiJMb2NhbCBkZW1vIikpLAogICAgZXJyb3JfbWF0Y2g6bm9kZXMuc29tZShuPT5uLm5hbWU9PT0iTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LiIpLAogICAgcmVxdWlyZWRfdGV4dDpub2Rlcy5zb21lKG49Pm4ubmFtZT09PShraW5kPT09InByb3RlY3RlZF9iZWZvcmUiPyJTaWduIGluIHJlcXVpcmVkLiI6CiAgICAgIGtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIj8iU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iOgogICAgICAiVXNlIFNpZ24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLiIpKSwKICAgIGludGVyYWN0aXZlX3JlZl9wcmVzZW50OkFycmF5LmlzQXJyYXkocz8ucmVmcykmJgogICAgICBzLnJlZnMuc29tZShuPT5BcnJheS5pc0FycmF5KG4uYWN0aW9ucykmJm4uYWN0aW9ucy5pbmNsdWRlcygiY2xpY2siKSkKICB9OwogIHJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M9e2tpbmQsLi4uZmxhZ3N9O3JldGFpbigpOwogIC8vIE9ubHkgcHJvdmlkZXIgcmVmcyBhbmQgZml4ZWQgcHVibGljLWxhYmVsIG1hdGNoZXMgc3Vydml2ZSB0aGlzIHJhdyBzbmFwc2hvdC4KICBvd24uZnJlc2hfcmVmcz1BcnJheS5pc0FycmF5KHM/LnJlZnMpP3MucmVmcy5maWx0ZXIobj0+dHlwZW9mIG4ucmVmPT09InN0cmluZyImJgogICAgL15wWzAtOV0rOlswLTldKyQvLnRlc3Qobi5yZWYpKS5tYXAobj0+bi5yZWYpOltdOwogIG93bi5wdWJsaWNfZGVjaXNpb249cHJvamVjdFB1YmxpY0RlY2lzaW9uKHJlc3VsdCk7CiAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICBpZihmbGFncy5lcnJvcl9tYXRjaClyZXR1cm4gZmFsc2U7CiAgaWYoIWZsYWdzLnN0YXR1c19va3x8IWZsYWdzLmNvbXBsZXRlKXJldHVybiBmYWxzZTsKICBpZihraW5kPT09ImdlbmVyaWNfY2hlY2tlZCIpIHsKICAgIGlmKG5vZGVzLnNvbWUobj0+bi5yb2xlPT09ImhlYWRpbmciJiZuLm5hbWU9PT0iUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyIpJiYKICAgICAgIG5vZGVzLnNvbWUobj0+bi5uYW1lPT09IlNpZ25lZCBpbi4gUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyBpcyBhdmFpbGFibGUuIikpCiAgICAgIHJlY29yZC5wcm90ZWN0ZWRfYWZ0ZXJfcGFnZT10cnVlOwogICAgcmV0dXJuIHRydWU7CiAgfQogIGNvbnN0IG9rPWZsYWdzLmhlYWRpbmdfbWF0Y2gmJmZsYWdzLnJlcXVpcmVkX3RleHQmJgogICAgKGtpbmQhPT0iYXBwbGljYXRpb24ifHxmbGFncy5pbnRlcmFjdGl2ZV9yZWZfcHJlc2VudCk7CiAgaWYob2smJmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIilyZWNvcmQucHJvdGVjdGVkX2FmdGVyX3BhZ2U9dHJ1ZTsKICByZXR1cm4gb2s7Cn0KYXN5bmMgZnVuY3Rpb24gbmF2aWdhdGVBbmRTbmFwc2hvdCh1cmwsa2luZCkgewogIGlmKGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfbmF2aWdhdGlvbiIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2Jyb3dzZXJfbmF2aWdhdGUoe3Nlc3Npb246b3duLnNlc3Npb24sCiAgICAgICAgdGFyZ2V0X2lkOm93bi50YXJnZXRfaWQsdGFiX2lkOm93bi50YWJfaWQsdXJsfSksCiAgICAgIHI9PnI/LmlzRXJyb3IhPT10cnVlKSkgewogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncykscj0+cGFnZShyLGtpbmQpKTsKICB9Cn0KYXN5bmMgZnVuY3Rpb24gZW50cnkoKSB7CiAgLy8gVGhlIGV4YWN0IERyaXZlci1vd25lZCBibGFuayBoYW5kbGVzIGFscmVhZHkgZXhpc3Qgd2hpbGUgdGhlMTgwcyBnYXRlIHdhaXRzLgogIC8vIFRoZXJlIGlzIG5vIG1vZGVsIHlpZWxkLCB0b29sLWRlc2NyaXB0aW9uIGxvYWQgb3IgcmViaW5kIGFmdGVyIHRoaXMgbWFya2VyLgogIGNvbnN0IG1hcmtlckNvbW1hbmQ9InB5dGhvbjMgLWMgIitzcShNQVJLRVIpKyIgIitzcShvd24ubGFiKSsiICIrCiAgICBzcShvd24uZ3VhcmRfcGlkKSsiICIrc3Eob3duLnNlcnZlcl9waWQpOwogIGF3YWl0IGNoZWNrZWQoInByZXBhcmVkX21hcmtlciIsCiAgICAoKT0+dG9vbHMuZXhlY19jb21tYW5kKHtjbWQ6bWFya2VyQ29tbWFuZCx3b3JrZGlyOm93bi53b3Jrc3BhY2UsCiAgICAgIHlpZWxkX3RpbWVfbXM6MTAwMDAsbWF4X291dHB1dF90b2tlbnM6NTAwfSkscj0+ewogICAgICByZWNvcmQubG9jYWxfdG9vbF9yZWNlaXB0cy5wdXNoKHtsYWJlbDoibWFya2VyIiwKICAgICAgICBleGl0OnR5cGVvZiByLmV4aXRfY29kZT09PSJudW1iZXIiP3IuZXhpdF9jb2RlOm51bGwsCiAgICAgICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGx9KTtyZXRhaW4oKTsKICAgICAgaWYoci5zZXNzaW9uX2lkIT09dW5kZWZpbmVkKWNsZWFudXBFcnJvcigib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCIpOwogICAgICByZXR1cm4gci5leGl0X2NvZGU9PT0wJiZyLnNlc3Npb25faWQ9PT11bmRlZmluZWQmJgogICAgICAgIEpTT04ucGFyc2Uoci5vdXRwdXQpLnByZXBhcmVkX21hcmtlcl93cml0dGVuPT09dHJ1ZTsKICAgIH0pOwogIGNvbnN0IHJlYWR5RGVhZGxpbmU9RGF0ZS5ub3coKSszMDAwMDsKICB3aGlsZShyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwmJm93bi5maXh0dXJlX3JlYWR5IT09dHJ1ZSYmRGF0ZS5ub3coKTxyZWFkeURlYWRsaW5lKQogICAgYXdhaXQgcG9sbENvbnRyb2xsZXIoKTsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwmJm93bi5maXh0dXJlX3JlYWR5IT09dHJ1ZSkKICAgIGxhdGNoKCJmaXh0dXJlX3JlYWR5X3VuY29uZmlybWVkIixEYXRlLm5vdygpKTsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpe2F3YWl0IGNsZWFudXAoKTtyZXR1cm47fQogIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIE9ORSBpbW1lZGlhdGUgcHJlLW5hdmlnYXRpb24gcG9sbCwgbm8gUlAgSFRUUCBwcm9iZS4KICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpe2F3YWl0IGNsZWFudXAoKTtyZXR1cm47fQogIGF3YWl0IG5hdmlnYXRlQW5kU25hcHNob3QoImh0dHA6Ly9sb2NhbGhvc3Q6MzAwMC9wcm90ZWN0ZWQiLCJwcm90ZWN0ZWRfYmVmb3JlIik7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBuYXZpZ2F0ZUFuZFNuYXBzaG90KCJodHRwOi8vbG9jYWxob3N0OjMwMDAvIiwiYXBwbGljYXRpb24iKTsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwpYXdhaXQgcG9sbENvbnRyb2xsZXIoKTsgLy8gT05FIHBvc3QtZW50cnkgcG9sbC4KICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwpIHsKICAgIG93bi5waGFzZT0iYnJvd3Nlcl9kZWNpc2lvbiI7CiAgICBzdG9yZSgiZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIixvd24pOwogIH0KICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpYXdhaXQgY2xlYW51cCgpOwp9CmFzeW5jIGZ1bmN0aW9uIGNvbnRpbnVhdGlvbigpIHsKICBjb25zdCBzcGVjPW93bi5uZXh0X2RlY2lzaW9uOwogIGNvbnN0IGFsbG93ZWRLaW5kcz1bImNsaWNrIiwidXNlcm5hbWUiLCJwYXNzd29yZCIsInNuYXBzaG90IiwicHJvdGVjdGVkX2FmdGVyIl07CiAgaWYoIXNwZWN8fCFhbGxvd2VkS2luZHMuaW5jbHVkZXMoc3BlYy5raW5kKSkgewogICAgbGF0Y2goImJyb3dzZXJfZGVjaXNpb25fdW5jb25maXJtZWQiLERhdGUubm93KCkpO2F3YWl0IGNsZWFudXAoKTtyZXR1cm47CiAgfQogIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBpZihzcGVjLmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIikgewogICAgLy8gUmVhZCB0aGUgZnJlc2ggY2FsbGJhY2stZm9sbG93aW5nIHByb3RlY3RlZCBwYWdlOyBkbyBub3Qgc2VuZCBhbm90aGVyIFJQIHJlcXVlc3QuCiAgICBhd2FpdCBjaGVja2VkKCJicm93c2VyX3NuYXBzaG90IiwKICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fZ2V0X2Jyb3dzZXJfc3RhdGUoc25hcHNob3RBcmdzKSxyPT5wYWdlKHIsInByb3RlY3RlZF9hZnRlciIpKTsKICB9IGVsc2UgaWYoc3BlYy5raW5kPT09InNuYXBzaG90IikgewogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncyksCiAgICAgIHI9PnBhZ2UociwiZ2VuZXJpY19jaGVja2VkIikmJnB1YmxpY1ByZWRpY2F0ZShyLHNwZWMpKTsKICB9IGVsc2UgewogICAgaWYodHlwZW9mIHNwZWMucmVmIT09InN0cmluZyJ8fCFBcnJheS5pc0FycmF5KG93bi5mcmVzaF9yZWZzKXx8CiAgICAgICAhb3duLmZyZXNoX3JlZnMuaW5jbHVkZXMoc3BlYy5yZWYpKSB7CiAgICAgIGxhdGNoKCJicm93c2VyX2RlY2lzaW9uX3VuY29uZmlybWVkIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuOwogICAgfQogICAgb3duLmZyZXNoX3JlZnM9W107c3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsgLy8gU2luZ2xlLXVzZSBzbmFwc2hvdCByZWYuCiAgICBjb25zdCBhcmdzPXtzZXNzaW9uOm93bi5zZXNzaW9uLHRhcmdldF9pZDpvd24udGFyZ2V0X2lkLHRhYl9pZDpvd24udGFiX2lkLHJlZjpzcGVjLnJlZn07CiAgICBjb25zdCBvcGVyYXRpb249c3BlYy5raW5kPT09ImNsaWNrIj8KICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fYnJvd3Nlcl9jbGljayh7Li4uYXJncyxpbnB1dF9yb3V0ZToiZG9tX2V2ZW50In0pOgogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19icm93c2VyX3R5cGUoey4uLmFyZ3MscmVwbGFjZTp0cnVlLAogICAgICAgIHRleHQ6c3BlYy5raW5kPT09InVzZXJuYW1lIj8iYWRtaW4iOmxvYWQoImQwMV9mcmVzaF9wYXNzd29yZF9pbnB1dCIpfSk7CiAgICBpZihzcGVjLmtpbmQ9PT0icGFzc3dvcmQiJiYodHlwZW9mIGxvYWQoImQwMV9mcmVzaF9wYXNzd29yZF9pbnB1dCIpIT09InN0cmluZyJ8fAogICAgICAgIS9eW0EtWmEtejAtOV8tXXszMCwxMjh9JC8udGVzdChsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKSkpKSB7CiAgICAgIGxhdGNoKCJicm93c2VyX2RlY2lzaW9uX3VuY29uZmlybWVkIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuOwogICAgfQogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9pbnB1dCIsb3BlcmF0aW9uLHI9PnsKICAgICAgY29uc3Qgcz1yPy5zdHJ1Y3R1cmVkQ29udGVudDsKICAgICAgcmV0dXJuIHI/LmlzRXJyb3IhPT10cnVlJiZbImNvbmZpcm1lZCIsInVudmVyaWZpYWJsZSJdLmluY2x1ZGVzKHM/LmVmZmVjdCk7CiAgICB9KTsgLy8gRGlzcGF0Y2ggYWxvbmUgbmV2ZXIgZWFybnMgYW4gYXBwbGljYXRpb24gb3V0Y29tZSBvciBqb3VybmV5IGNyZWRpdC4KICAgIGlmKHNwZWMua2luZD09PSJwYXNzd29yZCIpc3RvcmUoImQwMV9mcmVzaF9wYXNzd29yZF9pbnB1dCIsbnVsbCk7CiAgICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwpCiAgICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncyksCiAgICAgICAgcj0+cGFnZShyLCJnZW5lcmljX2NoZWNrZWQiKSYmcHVibGljUHJlZGljYXRlKHIsc3BlYykpOwogIH0KICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwpYXdhaXQgcG9sbENvbnRyb2xsZXIoKTsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpYXdhaXQgY2xlYW51cCgpOwogIGVsc2UgaWYoc3BlYy5raW5kPT09InByb3RlY3RlZF9hZnRlciIpIHsKICAgIHJlY29yZC5jbGVhbnVwX3JlcXVlc3RlZF93YWxsX21zPURhdGUubm93KCk7cmV0YWluKCk7CiAgICBhd2FpdCBjbGVhbnVwKCk7CiAgfQp9CmZ1bmN0aW9uIHByb2plY3RQdWJsaWNEZWNpc2lvbihyZXN1bHQpIHsKICBjb25zdCBzPXJlc3VsdD8uc3RydWN0dXJlZENvbnRlbnQsbm9kZXM9QXJyYXkuaXNBcnJheShzPy5jb250ZW50X3JlZnMpP3MuY29udGVudF9yZWZzOltdOwogIGNvbnN0IGxhYmVscz1bCiAgICBbImhlYWRpbmciLCJMb2NhbCBkZW1vIl0sWyJoZWFkaW5nIiwiU2lnbiBpbiB0byBjb250aW51ZSB0byBMb2NhbCBkZW1vIl0sCiAgICBbImhlYWRpbmciLCJMb2NhbCBkZW1vIHdhbnRzIHRvIHVzZSB5b3VyIHJpQXV0aCBhY2NvdW50Il0sCiAgICBbImhlYWRpbmciLCJDb250aW51ZSB0byBMb2NhbCBkZW1vPyJdLFsiaGVhZGluZyIsIlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiXSwKICAgIFsiYnV0dG9uIiwiU2lnbiBpbiJdLFsiYnV0dG9uIiwiQWxsb3ciXSxbImJ1dHRvbiIsIkNvbnRpbnVlIl0sCiAgICBbInRleHRib3giLCJVc2VybmFtZSJdLFsidGV4dGJveCIsIlBhc3N3b3JkIl0sCiAgICBbInRleHRib3giLCJBdXRoZW50aWNhdG9yIG9yIHJlY292ZXJ5IGNvZGUgKGlmIGVuYWJsZWQpIl0sCiAgICBbInN0YXRpY3RleHQiLCJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZhaWxhYmxlLiJdCiAgXTsKICBjb25zdCB2YWxpZD1yZXN1bHQ/LmlzRXJyb3IhPT10cnVlJiZzPy5zdGF0dXM9PT0ib2siJiZzPy5zbmFwc2hvdD8uY29tcGxldGU9PT10cnVlJiYKICAgICFub2Rlcy5zb21lKG49Pm4ubmFtZT09PSJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3QuIik7CiAgcmV0dXJuIHsKICAgIG9ic2VydmVkOnZhbGlkP2xhYmVscy5maWx0ZXIoKFtyb2xlLG5hbWVdKT0+CiAgICAgIG5vZGVzLnNvbWUobj0+bi5yb2xlPT09cm9sZSYmbi5uYW1lPT09bmFtZSkpLm1hcCgoW3JvbGUsbmFtZV0pPT4oe3JvbGUsbmFtZX0pKTpbXSwKICAgIHJlZnM6dmFsaWQmJkFycmF5LmlzQXJyYXkocz8ucmVmcyk/cy5yZWZzLmZsYXRNYXAobj0+ewogICAgICBjb25zdCBwYWlyPWxhYmVscy5maW5kKChbcm9sZSxuYW1lXSk9Pm4/LnJvbGU9PT1yb2xlJiZuLm5hbWU9PT1uYW1lKSxyZWY9bj8ucmVmOwogICAgICBpZighcGFpcnx8dHlwZW9mIHJlZiE9PSJzdHJpbmcifHwhL15wWzAtOV0rOlswLTldKyQvLnRlc3QocmVmKSlyZXR1cm4gW107CiAgICAgIGNvbnN0IFtyb2xlLG5hbWVdPXBhaXI7CiAgICAgIGNvbnN0IGFjdGlvbj1yb2xlPT09ImJ1dHRvbiI/ImNsaWNrIjoKICAgICAgICByb2xlPT09InRleHRib3giJiZbIlVzZXJuYW1lIiwiUGFzc3dvcmQiXS5pbmNsdWRlcyhuYW1lKT8idHlwZSI6bnVsbDsKICAgICAgaWYoYWN0aW9uPT09bnVsbHx8IUFycmF5LmlzQXJyYXkobi5hY3Rpb25zKXx8IW4uYWN0aW9ucy5pbmNsdWRlcyhhY3Rpb24pKXJldHVybiBbXTsKICAgICAgcmV0dXJuIFt7cm9sZSxuYW1lLGFjdGlvbixyZWZ9XTsKICAgIH0pOltdCiAgfTsKfQpmdW5jdGlvbiBwdWJsaWNQcmVkaWNhdGUocmVzdWx0LHNwZWMpIHsKICAvLyBSb290IG11c3QgcGluIG9uZSBwcmludGVkIHB1YmxpYyBleHBlY3RlZCBsYWJlbC9yb2xlIGJlZm9yZSB0aGF0IGFjdGlvbi4KICAvLyBObyByYXcgZmllbGQgdmFsdWUsIFVSTCwgc3ViamVjdCwgcXVlcnkgb3IgYXJiaXRyYXJ5IHJlZ2V4IHByZWRpY2F0ZSBpcyBhbGxvd2VkLgogIHJldHVybiBwcm9qZWN0UHVibGljRGVjaXNpb24ocmVzdWx0KS5vYnNlcnZlZC5zb21lKG49PgogICAgbi5yb2xlPT09c3BlYy5leHBlY3RlZF9yb2xlJiZuLm5hbWU9PT1zcGVjLmV4cGVjdGVkX2xhYmVsKTsKfQp0cnkgewogIGlmKG93bi5waGFzZT09PSJwcmVwYXJlZF9lbnRyeSIpYXdhaXQgZW50cnkoKTtlbHNlIGF3YWl0IGNvbnRpbnVhdGlvbigpOwogIC8vIERvIG5vdCBjYXJyeSB1bnZhbGlkYXRlZCBwYXJ0aWFsIGNvbnRyb2xsZXIgdGV4dCBhY3Jvc3MgdGhpcyBjZWxsIGJvdW5kYXJ5LgogIHdoaWxlKGNvbnRyb2xsZXJCdWZmZXIubGVuZ3RoPjAmJnJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbCkgewogICAgY29uc3QgYmVmb3JlUG9sbFdhbGw9RGF0ZS5ub3coKTsKICAgIGlmKGJlZm9yZVBvbGxXYWxsPj1vd24uc3RhcnRfZXBvY2hfbXMrODQwMDAwKSB7CiAgICAgIGxhdGNoKCJicm93c2VyX2FjdGl2ZV9kZWFkbGluZSIsYmVmb3JlUG9sbFdhbGwpO2JyZWFrOwogICAgfQogICAgaWYoY29udHJvbGxlckpvaW5lZHx8IWNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlKSB7CiAgICAgIGxhdGNoKCJjb250cm9sbGVyX29ic2VydmF0aW9uX2ludmFsaWQiLGJlZm9yZVBvbGxXYWxsKTticmVhazsKICAgIH0KICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIFNhbWUgb3duZWQgc2Vzc2lvbjsgb2JzZXJ2ZXIgcmV0YWlucyByZWNlaXB0IGZpcnN0LgogICAgY29uc3QgYWZ0ZXJQb2xsV2FsbD1EYXRlLm5vdygpOwogICAgaWYoYWZ0ZXJQb2xsV2FsbD49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkKICAgICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixhZnRlclBvbGxXYWxsKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfSBjYXRjaCB7CiAgbGF0Y2goImNvbXBvc2VkX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7CiAgYXdhaXQgY2xlYW51cCgpOwp9CmlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCkgewogIHRleHQoe3Jlc3VsdDoiZmFpbGVkIixmaXJzdF9mYWlsdXJlOnJlY29yZC5maXJzdF9mYWlsdXJlLAogICAgdW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uOnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sCiAgICBkaWFnbm9zdGljX2Vycm9yczpyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMsY2xlYW51cF9lcnJvcnM6cmVjb3JkLmNsZWFudXBfZXJyb3JzLAogICAgZmluYWxfYWJzZW5jZTpyZWNvcmQuZmluYWxfYWJzZW5jZSxjb250cm9sbGVyX2V4aXQ6cmVjb3JkLmNvbnRyb2xsZXJfZXhpdCwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogICAgcmVzb3VyY2VfcmVsZWFzZV9wcm92ZW46cmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKX0pOwp9IGVsc2UgewogIHRleHQoe3Jlc3VsdDoiYnJvd3Nlcl9kZWNpc2lvbl9vYnNlcnZlZF9vbmx5IixwYWdlX2ZsYWdzOnJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M/P251bGwsCiAgICBwdWJsaWNfZGVjaXNpb246b3duLnB1YmxpY19kZWNpc2lvbj8/bnVsbCwKICAgIGpvdXJuZXlfY3JlZGl0OmZhbHNlLGNsZWFudXBfZXJyb3JzOnJlY29yZC5jbGVhbnVwX2Vycm9ycywKICAgIHJlc291cmNlX3JlbGVhc2VfcHJvdmVuOnJlY29yZC5jbGVhbnVwX3N0YXJ0ZWQ9PT10cnVlJiZyZWNvcmQuZmluYWxfYWJzZW5jZSE9PW51bGwmJgogICAgICAhcmVjb3JkLmNsZWFudXBfZXJyb3JzLnNvbWUodj0+WwogICAgICAgICJjaGlsZF9yZWFwX2luY29tcGxldGUiLCJvd25lZF9yZXNvdXJjZV9hYnNlbmNlX3VucHJvdmVuIiwKICAgICAgICAib3duZWRfcmVhZGJhY2tfdW5hdmFpbGFibGUiLCJvd25lZF9sb2NhbF9jb21tYW5kX3Vuam9pbmVkIl0uaW5jbHVkZXModikpLAogICAgd2hvbGVfY2xlYW51cF93aXRoaW42MF9wcm92ZW46ZmFsc2V9KTsKfQo=",
  "baseline_b64": "Ly8gUHJvc3BlY3RpdmUgT05FIGZ1bmN0aW9ucy5leGVjIGNlbGw7IE5PVCBleGVjdXRlZCBpbiB0aGlzIGRlc2lnbiBwaGFzZS4KY29uc3QgQ0xPQ0s9ImltcG9ydCBqc29uLHRpbWVcbnByaW50KGpzb24uZHVtcHMoeydtb25vdG9uaWNfbnMnOnN0cih0aW1lLm1vbm90b25pY19ucygpKSwnd2FsbF9lcG9jaF9ucyc6c3RyKHRpbWUudGltZV9ucygpKX0pKVxuIjsKY29uc3QgU1RPUD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN5cyx0aW1lXG5jPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMV0pXG5jbG9jaz17J21vbm90b25pY19ucyc6c3RyKHRpbWUubW9ub3RvbmljX25zKCkpLCd3YWxsX2Vwb2NoX25zJzpzdHIodGltZS50aW1lX25zKCkpfVxucmVjb3JkPXsnc2NoZW1hJzoncmlhdXRoLmQwMS1maXJzdC1vYnNlcnZhdGlvbi92MScsJ2ZpcnN0X2ZhaWx1cmUnOmNbJ2ZpcnN0X2ZhaWx1cmUnXSwnZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyc6Y1snZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyddLCdmaXJzdF9ldmVudF9wcm92ZW4nOkZhbHNlLCdjbG9jayc6Y2xvY2t9XG5ldmVudF9zdGF0ZT0nd3JpdGVfdW5jb25maXJtZWQnXG50cnk6XG4gICAgZmQ9b3Mub3BlbihwYXRobGliLlBhdGgoY1snZXZlbnRfb3V0J10pLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApXG4gICAgd2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgICAgIGYud3JpdGUoanNvbi5kdW1wcyhyZWNvcmQsc29ydF9rZXlzPVRydWUpKydcXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSlcbiAgICBldmVudF9zdGF0ZT0nd3JpdHRlbidcbmV4Y2VwdCBPU0Vycm9yOnBhc3NcbiMgQXR0ZW1wdCBjbG9jayBwZXJzaXN0ZW5jZSBCRUZPUkUgbWFya2VyL3N0YXRlL2J1ZGdldCBjb21wYXJpc29ucy5cbiMgQSBtZXRhZGF0YSBlcnJvciBkb2VzIG5vdCBwcmV2ZW50IHRoZSBvcmlnaW5hbCBlc3NlbnRpYWwgc3RvcCBwcm90b2NvbC5cbmxhYj1wYXRobGliLlBhdGgoY1snbGFiJ10pO21hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbnRyeTpcbiAgICBpZiBsYWIuZXhpc3RzKCk6XG4gICAgICAgIGluZm89bGFiLmxzdGF0KClcbiAgICAgICAgaWYgbm90IHN0YXQuU19JU0RJUihpbmZvLnN0X21vZGUpIG9yIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpIT0wbzcwMCBvciBpbmZvLnN0X3VpZCE9b3MuZ2V0dWlkKCk6XG4gICAgICAgICAgICBtYXJrZXJfc3RhdGU9J293bmVyc2hpcF91bmtub3duJ1xuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgbWFya2VyX3N0YXRlPSdzdG9wX3JlcXVlc3RlZCdcbiAgICAgICAgICAgIGZvciBuYW1lIGluICgoJ3VpLWZhaWx1cmUnLCdzdG9wJykgaWYgY1snZmlyc3RfZmFpbHVyZSddIGlzIG5vdCBOb25lIGVsc2UgKCdzdG9wJywpKTpcbiAgICAgICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgICAgIGZkPW9zLm9wZW4obGFiL25hbWUsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbiAgICAgICAgICAgICAgICAgICAgb3MuY2xvc2UoZmQpXG4gICAgICAgICAgICAgICAgZXhjZXB0IEZpbGVOb3RGb3VuZEVycm9yOm1hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbiAgICAgICAgICAgICAgICBleGNlcHQgRmlsZUV4aXN0c0Vycm9yOnBhc3NcbmV4Y2VwdCBPU0Vycm9yOm1hcmtlcl9zdGF0ZT0nc3RvcF91bmNvbmZpcm1lZCdcbnByaW50KGpzb24uZHVtcHMoeydjbG9jayc6Y2xvY2ssJ2V2ZW50X3N0YXRlJzpldmVudF9zdGF0ZSwnbWFya2VyX3N0YXRlJzptYXJrZXJfc3RhdGV9KSlcbiI7CmNvbnN0IFJFQURCQUNLPSJpbXBvcnQganNvbixvcyxwYXRobGliLHN0YXQsc3VicHJvY2VzcyxzeXMsdGltZVxuYz1qc29uLmxvYWRzKHN5cy5hcmd2WzFdKVxuY2hpbGRyZW49Tm9uZTtoZWxwZXJfcGlkPWNbJ2hlbHBlcl9waWQnXTtvdXRlcl9vYnNlcnZhdGlvbj1Ob25lO3Byb2plY3Rpb249J3VuYXZhaWxhYmxlJ1xuZGVmIGZpbml0ZV9vYnNlcnZhdGlvbih2YWx1ZSk6XG4gICAgaWYgdHlwZSh2YWx1ZSkgaXMgbm90IGRpY3Qgb3Igc2V0KHZhbHVlKSE9IHsnb2JzZXJ2ZWQnLCdkaWFnbm9zdGljJ30gb3IgdHlwZSh2YWx1ZVsnb2JzZXJ2ZWQnXSkgaXMgbm90IGJvb2w6XG4gICAgICAgIHJldHVybiBOb25lXG4gICAgZGlhZ25vc3RpYz12YWx1ZVsnZGlhZ25vc3RpYyddXG4gICAgaWYgZGlhZ25vc3RpYyBpcyBOb25lOlxuICAgICAgICByZXR1cm4geydvYnNlcnZlZCc6dmFsdWVbJ29ic2VydmVkJ10sJ2RpYWdub3N0aWMnOk5vbmV9XG4gICAgaWYgbm90IHZhbHVlWydvYnNlcnZlZCddIG9yIHR5cGUoZGlhZ25vc3RpYykgaXMgbm90IGRpY3Qgb3Igc2V0KGRpYWdub3N0aWMpIT0geydzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnfTpcbiAgICAgICAgcmV0dXJuIE5vbmVcbiAgICBzaXRlcz0oJ2hhbmRsZXInLCdzZXJ2ZXInLCdtYWluJylcbiAgICBraW5kcz0oJ0F0dHJpYnV0ZUVycm9yJywnVHlwZUVycm9yJywnVmFsdWVFcnJvcicsJ0tleUVycm9yJywnT1NFcnJvcicsJ0Jyb2tlblBpcGVFcnJvcicsJ0Nvbm5lY3Rpb25SZXNldEVycm9yJywnVGltZW91dEVycm9yJywnb3RoZXInKVxuICAgIGZ1bmN0aW9ucz0oJ0hlYWRlclJlYWRlci5yZWFkbGluZScsJ0RlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0JywnRGVtby5iZWdpbicsJ0RlbW8uY2FsbGJhY2snLCdEZW1vLmludm9rZScsJ0hhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0JywnSGFuZGxlci5zZW5kX2Vycm9yJywnSGFuZGxlci5nZXQnLCdIYW5kbGVyLnJlcGx5JywnbWFpbicpXG4gICAgc2l0ZSxraW5kLGZ1bmN0aW9uLGxpbmU9KGRpYWdub3N0aWNba10gZm9yIGsgaW4gKCdzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnKSlcbiAgICBpZiB0eXBlKHNpdGUpIGlzIG5vdCBzdHIgb3Igc2l0ZSBub3QgaW4gc2l0ZXMgb3IgdHlwZShraW5kKSBpcyBub3Qgc3RyIG9yIGtpbmQgbm90IGluIGtpbmRzOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIGlmIG5vdCAoKGZ1bmN0aW9uIGlzIE5vbmUgYW5kIGxpbmUgaXMgTm9uZSkgb3IgKHR5cGUoZnVuY3Rpb24pIGlzIHN0ciBhbmQgZnVuY3Rpb24gaW4gZnVuY3Rpb25zIGFuZCB0eXBlKGxpbmUpIGlzIGludCBhbmQgMTw9bGluZTw9MTAyNCkpOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIHJldHVybiB7J29ic2VydmVkJzpUcnVlLCdkaWFnbm9zdGljJzp7J3NpdGUnOnNpdGUsJ2V4Y2VwdGlvbl9jbGFzcyc6a2luZCwnb3duX2Z1bmN0aW9uJzpmdW5jdGlvbiwnb3duX2xpbmUnOmxpbmV9fVxub3V0ZXI9cGF0aGxpYi5QYXRoKGNbJ291dGVyX291dCddKVxuaWYgb3V0ZXIuaXNfZmlsZSgpOlxuICAgIGZkPW9zLm9wZW4ob3V0ZXIsb3MuT19SRE9OTFl8b3MuT19OT0ZPTExPV3xvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbz1vcy5mc3RhdChmZClcbiAgICAgICAgaWYgbm90IChzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKT09MG82MDAgYW5kIDA8aW5mby5zdF9zaXplPD0yNjIxNDQpOlxuICAgICAgICAgICAgcmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIHdpdGggb3MuZmRvcGVuKGZkLCdyYicsY2xvc2VmZD1GYWxzZSkgYXMgc291cmNlOnJhd19ieXRlcz1zb3VyY2UucmVhZCgyNjIxNDUpXG4gICAgICAgIGlmIG5vdCAwPGxlbihyYXdfYnl0ZXMpPD0yNjIxNDQ6cmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIGRhdGE9anNvbi5sb2FkcyhyYXdfYnl0ZXMpXG4gICAgZmluYWxseTpvcy5jbG9zZShmZClcbiAgICBvdXRlcl9vYnNlcnZhdGlvbj1maW5pdGVfb2JzZXJ2YXRpb24oZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbicpKVxuICAgIHByb2plY3Rpb249ZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfb2JzZXJ2YXRpb25fcHJvamVjdGlvbicpXG4gICAgaWYgcHJvamVjdGlvbiBub3QgaW4gKCd1bmF2YWlsYWJsZScsJ3ZhbGlkJywnaW52YWxpZCcpOnByb2plY3Rpb249J2ludmFsaWQnXG4gICAgaWYgcHJvamVjdGlvbj09J3ZhbGlkJyBhbmQgb3V0ZXJfb2JzZXJ2YXRpb24gaXMgTm9uZTpwcm9qZWN0aW9uPSdpbnZhbGlkJ1xuICAgIGFsbG93ZWQ9eyd3aG9hbWknLCdkaXNjb3ZlcnknLCdjb25maWRlbnRpYWxfY2xpZW50X2NyZWF0ZScsJ29wZXJhdG9yX2xvZ2luJywnc2VydmVyJywnbWFpbnRlbmFuY2VfaW5pdCd9XG4gICAgc3RhcnRlZD1kYXRhLmdldCgnaGVscGVyX2ludm9jYXRpb25zJyk9PTEgYW5kIHR5cGUoZGF0YS5nZXQoJ2hlbHBlcl9waWQnKSkgaXMgaW50IGFuZCBkYXRhWydoZWxwZXJfcGlkJ10+MFxuICAgIGlmIHN0YXJ0ZWQ6YWxsb3dlZC5hZGQoJ2hlbHBlcicpXG4gICAgcmF3PWRhdGEuZ2V0KCdvd25lZF9jaGlsZF9leGl0cycpXG4gICAgaWYgaXNpbnN0YW5jZShyYXcsbGlzdCkgYW5kIGxlbihyYXcpPT1sZW4oYWxsb3dlZCkgYW5kIGFsbChpc2luc3RhbmNlKHYsZGljdCkgYW5kIHYuZ2V0KCduYW1lJykgaW4gYWxsb3dlZCBhbmQgdHlwZSh2LmdldCgncGlkJykpIGlzIGludCBhbmQgdlsncGlkJ10+MCBhbmQgdHlwZSh2LmdldCgnZXhpdCcpKSBpcyBpbnQgZm9yIHYgaW4gcmF3KSBhbmQge3ZbJ25hbWUnXSBmb3IgdiBpbiByYXd9PT1hbGxvd2VkIGFuZCBsZW4oe3ZbJ3BpZCddIGZvciB2IGluIHJhd30pPT1sZW4oYWxsb3dlZCkgYW5kIG5leHQodlsncGlkJ10gZm9yIHYgaW4gcmF3IGlmIHZbJ25hbWUnXT09J3NlcnZlcicpPT1jWydzZXJ2ZXJfcGlkJ10gYW5kICgoc3RhcnRlZCBhbmQgbmV4dCh2WydwaWQnXSBmb3IgdiBpbiByYXcgaWYgdlsnbmFtZSddPT0naGVscGVyJyk9PWRhdGFbJ2hlbHBlcl9waWQnXSBhbmQgKGhlbHBlcl9waWQgaXMgTm9uZSBvciBoZWxwZXJfcGlkPT1kYXRhWydoZWxwZXJfcGlkJ10pKSBvciAobm90IHN0YXJ0ZWQgYW5kIGhlbHBlcl9waWQgaXMgTm9uZSBhbmQgZGF0YS5nZXQoJ2hlbHBlcl9pbnZvY2F0aW9ucycpIGlzIE5vbmUgYW5kIGRhdGEuZ2V0KCdoZWxwZXJfcGlkJykgaXMgTm9uZSkpOlxuICAgICAgICBjaGlsZHJlbj1be2s6dltrXSBmb3IgayBpbiAoJ25hbWUnLCdwaWQnLCdleGl0Jyl9IGZvciB2IGluIHJhd11cbiAgICAgICAgaGVscGVyX3BpZD1kYXRhWydoZWxwZXJfcGlkJ10gaWYgc3RhcnRlZCBlbHNlIE5vbmVcbnBpZHM9bGlzdChjWydvd25lZF9waWRzJ10pXG5pZiBoZWxwZXJfcGlkIGlzIG5vdCBOb25lIGFuZCBoZWxwZXJfcGlkIG5vdCBpbiBwaWRzOnBpZHMuYXBwZW5kKGhlbHBlcl9waWQpXG5wPXN1YnByb2Nlc3MucnVuKFsnL2Jpbi9wcycsJy1wJywnLCcuam9pbihzdHIodikgZm9yIHYgaW4gcGlkcyksJy1vJywncGlkPSddLGNhcHR1cmVfb3V0cHV0PVRydWUsdGltZW91dD0zKVxucHNfa25vd249cC5yZXR1cm5jb2RlIGluICgwLDEpIGFuZCBhbGwodi5pc2RpZ2l0KCkgZm9yIHYgaW4gcC5zdGRvdXQuc3BsaXQoKSlcbnByZXNlbnQ9c2V0KGludCh2KSBmb3IgdiBpbiBwLnN0ZG91dC5zcGxpdCgpKSBpZiBwc19rbm93biBlbHNlIHNldCgpXG5wb3J0cz17fVxuZm9yIHBvcnQgaW4gKDkwMDAsMzAwMCk6XG4gICAgcD1zdWJwcm9jZXNzLnJ1bihbJy91c3Ivc2Jpbi9sc29mJywnLW5QJywnLXQnLCctaVRDUDonK3N0cihwb3J0KSwnLXNUQ1A6TElTVEVOJ10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpXG4gICAgcG9ydHNbc3RyKHBvcnQpXT1ub3QgYm9vbChwLnN0ZG91dC5zdHJpcCgpKSBpZiBwLnJldHVybmNvZGUgaW4gKDAsMSkgZWxzZSBOb25lXG5wcmludChqc29uLmR1bXBzKHsnY2xvY2snOnsnbW9ub3RvbmljX25zJzpzdHIodGltZS5tb25vdG9uaWNfbnMoKSksJ3dhbGxfZXBvY2hfbnMnOnN0cih0aW1lLnRpbWVfbnMoKSl9LCdvd25lZF9waWRzJzp7c3RyKHYpOih2IG5vdCBpbiBwcmVzZW50IGlmIHBzX2tub3duIGVsc2UgTm9uZSkgZm9yIHYgaW4gcGlkc30sJ3BvcnRzJzpwb3J0cywnbGFiX2Fic2VudCc6bm90IHBhdGhsaWIuUGF0aChjWydsYWInXSkuZXhpc3RzKCksJ293bmVkX2NoaWxkX2V4aXRzJzpjaGlsZHJlbiwnaGVscGVyX3BpZCc6aGVscGVyX3BpZCwndW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uJzpvdXRlcl9vYnNlcnZhdGlvbiwndW5leHBlY3RlZF9vYnNlcnZhdGlvbl9wcm9qZWN0aW9uJzpwcm9qZWN0aW9ufSkpXG4iOwpjb25zdCBNQVJLRVI9ImltcG9ydCBvcyxwYXRobGliLHN0YXQsc3lzXG5sYWI9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKTtndWFyZD1pbnQoc3lzLmFyZ3ZbMl0pO3NlcnZlcj1pbnQoc3lzLmFyZ3ZbM10pXG5pZiBndWFyZDw9MCBvciBzZXJ2ZXI8PTAgb3IgZ3VhcmQ9PXNlcnZlcjpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKVxuaW5mbz1sYWIubHN0YXQoKVxuaWYgbm90IChzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSk9PTBvNzAwIGFuZCBpbmZvLnN0X3VpZD09b3MuZ2V0dWlkKCkpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5vcy5raWxsKGd1YXJkLDApO29zLmtpbGwoc2VydmVyLDApXG5pZiAobGFiLydzdG9wJykuZXhpc3RzKCkgb3IgKGxhYi8ndWktZmFpbHVyZScpLmV4aXN0cygpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5mZD1vcy5vcGVuKGxhYi8nYnJvd3Nlci1wcmVwYXJlZCcsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbm9zLmNsb3NlKGZkKVxucHJpbnQoJ3tcInByZXBhcmVkX21hcmtlcl93cml0dGVuXCI6dHJ1ZX0nKVxuIjsKY29uc3QgUEVSU0lTVD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzeXNcbnBhdGg9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKVxucmVjb3JkPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMl0pXG4jIENvbnZlcnQgZGVjaW1hbCBzdHJpbmdzIGRpcmVjdGx5IHRvIFB5dGhvbiBpbnRlZ2VycywgYXZvaWRpbmcgSlMgTnVtYmVyIHJvdW5kaW5nLlxuZGVmIGNsb2Nrcyh2YWx1ZSk6XG4gICAgaWYgaXNpbnN0YW5jZSh2YWx1ZSxkaWN0KTpcbiAgICAgICAgZm9yIGssdiBpbiBsaXN0KHZhbHVlLml0ZW1zKCkpOlxuICAgICAgICAgICAgaWYgayBpbiAoJ21vbm90b25pY19ucycsJ3dhbGxfZXBvY2hfbnMnKSBhbmQgaXNpbnN0YW5jZSh2LHN0cik6XG4gICAgICAgICAgICAgICAgYXNzZXJ0IHYuaXNkZWNpbWFsKCkgYW5kIGxlbih2KTw9MTkgYW5kIDA8PWludCh2KTwyKio2M1xuICAgICAgICAgICAgICAgIHZhbHVlW2tdPWludCh2KVxuICAgICAgICAgICAgZWxzZTpjbG9ja3ModilcbiAgICBlbGlmIGlzaW5zdGFuY2UodmFsdWUsbGlzdCk6XG4gICAgICAgIGZvciB2IGluIHZhbHVlOmNsb2Nrcyh2KVxuY2xvY2tzKHJlY29yZClcbmZkPW9zLm9wZW4ocGF0aCxvcy5PX1dST05MWXxvcy5PX0NSRUFUfG9zLk9fRVhDTHxvcy5PX05PRk9MTE9XLDBvNjAwKVxud2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgZi53cml0ZShqc29uLmR1bXBzKHJlY29yZCxzb3J0X2tleXM9VHJ1ZSxpbmRlbnQ9MikrJ1xcbicpO2YuZmx1c2goKTtvcy5mc3luYyhmLmZpbGVubygpKVxucHJpbnQoanNvbi5kdW1wcyh7J3dyaXR0ZW5fZXhjbHVzaXZlJzpUcnVlfSkpXG4iOwpjb25zdCBvd249bG9hZCgiZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIik7CmNvbnN0IE9CU0tFWT0iZDAxX2ltbWVkaWF0ZV9vYnNlcnZhdGlvbl9yZWNvcmQiOwppZiAoIW93biB8fCBvd24ucnVudGltZV9yZWxlYXNlIT09dHJ1ZSB8fCBvd24uZHJpdmVyX293bmVkIT09dHJ1ZSB8fAogICAgb3duLmV4YWN0X2JvdW5kIT09dHJ1ZSB8fCBvd24uc2luZ2xlX2NvbnRyb2xsZXIhPT10cnVlIHx8CiAgICAhWyJwcmVwYXJlZF9lbnRyeSIsImJyb3dzZXJfZGVjaXNpb24iXS5pbmNsdWRlcyhvd24ucGhhc2UpIHx8CiAgICAob3duLnBoYXNlPT09InByZXBhcmVkX2VudHJ5IiYmKG93bi5wcmVwYXJlZF9nYXRlIT09dHJ1ZXx8b3duLmhlbHBlcl9waWQhPT1udWxsfHwKICAgICAgb3duLmZpeHR1cmVfcmVhZHk9PT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mPT09dHJ1ZSkpIHx8CiAgICAob3duLnBoYXNlPT09ImJyb3dzZXJfZGVjaXNpb24iJiYob3duLmZpeHR1cmVfcmVhZHkhPT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mIT09dHJ1ZSkpIHx8CiAgICBvd24uZnJlc2hfcGF0aHNfcHJlZmxpZ2h0IT09dHJ1ZSB8fAogICAgIU51bWJlci5pc1NhZmVJbnRlZ2VyKG93bi5zdGFydF9lcG9jaF9tcykgfHwgdHlwZW9mIG93bi53b3Jrc3BhY2UhPT0ic3RyaW5nIiB8fAogICAgbmV3IFNldChbb3duLmd1YXJkX3BpZCxvd24uc2VydmVyX3BpZCxvd24uYnJvd3Nlcl9waWQsCiAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSkuc2l6ZSE9PShvd24uaGVscGVyX3BpZD09PW51bGw/Mzo0KSB8fAogICAgIVtvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCxvd24uZXhlY19zZXNzaW9uLAogICAgICAgLi4uKG93bi5oZWxwZXJfcGlkPT09bnVsbD9bXTpbb3duLmhlbHBlcl9waWRdKV0KICAgICAgIC5ldmVyeSh2PT5OdW1iZXIuaXNTYWZlSW50ZWdlcih2KSYmdj4wKSB8fAogICAgIVtvd24uc2Vzc2lvbixvd24udGFyZ2V0X2lkLG93bi50YWJfaWQsb3duLmxhYixvd24ub3V0ZXJfb3V0LAogICAgICAgb3duLmV2ZW50X291dCxvd24uY2xlYW51cF9vdXRdLmV2ZXJ5KHY9PnR5cGVvZiB2PT09InN0cmluZyImJnYubGVuZ3RoPjApKSB7CiAgdGV4dCh7cHJvcG9zYWxfcmVmdXNlZDoiZnJlc2hfb3duZWRfY29udGV4dF9yZXF1aXJlZCJ9KTtleGl0KCk7Cn0KY29uc3QgcHJpb3I9bG9hZChPQlNLRVkpOwppZihvd24uY2xvc2VkPT09dHJ1ZXx8cHJpb3I/LmNsZWFudXBfc3RhcnRlZD09PXRydWUpewogIHRleHQoe3Byb3Bvc2FsX3JlZnVzZWQ6ImZpeHR1cmVfYWxyZWFkeV9zdG9wcGVkIixyZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpmYWxzZX0pO2V4aXQoKTsKfQpjb25zdCByZWNvcmQ9cHJpb3I/P3sKICBzY2hlbWE6InJpYXV0aC5kMDEtaW1tZWRpYXRlLW9ic2VydmF0aW9uLWNsZWFudXAvdjEiLAogIGZpcnN0X2ZhaWx1cmU6bnVsbCxmaXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOm51bGwsCiAgZmlyc3RfZXZlbnRfcHJvdmVuOmZhbHNlLHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogIHVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbjpudWxsLGRpYWdub3N0aWNfZXJyb3JzOltdLGNsZWFudXBfc3RhcnRlZDpmYWxzZSwKICBmaXJzdF9jbG9jazpudWxsLG9ic2VydmF0aW9uX3JlY2VpcHRzOltdLGNvbnRyb2xsZXJfb2JzZXJ2YXRpb25zOltdLGxvY2FsX3Rvb2xfcmVjZWlwdHM6W10sYWN0aW9uczpbXSxjbGVhbnVwX2Vycm9yczpbXSwKICBmaW5hbF9hYnNlbmNlOm51bGwsb3duZWRfY2hpbGRfZXhpdHM6bnVsbCxjb250cm9sbGVyX2V4aXQ6bnVsbCwKICBmaXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zOm51bGwsbGF0Y2hfdG9fYWJzZW5jZV9tczpudWxsCn07CmNvbnN0IHNxPXM9PiInIitTdHJpbmcocykucmVwbGFjZSgvJy9nLCInXFwnJyIpKyInIjsKY29uc3QgcmV0YWluPSgpPT5zdG9yZShPQlNLRVkscmVjb3JkKTsKY29uc3QgY2xlYW51cEVycm9yPWxhYmVsPT57CiAgaWYoIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5pbmNsdWRlcyhsYWJlbCkpcmVjb3JkLmNsZWFudXBfZXJyb3JzLnB1c2gobGFiZWwpOwogIHJldGFpbigpOwp9Owpjb25zdCBsb2NhbD1hc3luYyhzb3VyY2UsYXJnKT0+ewogIGNvbnN0IGNtZD0icHl0aG9uMyAtYyAiK3NxKHNvdXJjZSkrKGFyZz09PXVuZGVmaW5lZD8iIjoiICIrc3EoSlNPTi5zdHJpbmdpZnkoYXJnKSkpOwogIGNvbnN0IHI9YXdhaXQgdG9vbHMuZXhlY19jb21tYW5kKHtjbWQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgeWllbGRfdGltZV9tczoxMDAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7CiAgICBsYWJlbDpzb3VyY2U9PT1DTE9DSz8iY2xvY2siOnNvdXJjZT09PVNUT1A/InN0b3AiOnNvdXJjZT09PVJFQURCQUNLPyJyZWFkYmFjayI6InVua25vd24iLAogICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGwKICB9KTtyZXRhaW4oKTsgLy8gTnVtZXJpYyBjb2xsZWN0b3IgcmVjZWlwdCBCRUZPUkUgY29tcGFyaXNvbnMuCiAgaWYoci5zZXNzaW9uX2lkIT09dW5kZWZpbmVkKSB7CiAgICBjbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTt0aHJvdyBuZXcgRXJyb3IoImxvY2FsX3BlbmRpbmciKTsKICB9CiAgaWYoci5leGl0X2NvZGUhPT0wKXRocm93IG5ldyBFcnJvcigibG9jYWxfZmFpbGVkIik7CiAgcmV0dXJuIEpTT04ucGFyc2Uoci5vdXRwdXQpOwp9OwpmdW5jdGlvbiBmaW5pdGVPYnNlcnZhdGlvbih2YWx1ZSkgewogIGNvbnN0IGV4YWN0PSh2LGtleXMpPT52IT09bnVsbCYmdHlwZW9mIHY9PT0ib2JqZWN0IiYmIUFycmF5LmlzQXJyYXkodikmJgogICAgT2JqZWN0LmtleXModikubGVuZ3RoPT09a2V5cy5sZW5ndGgmJmtleXMuZXZlcnkoaz0+T2JqZWN0Lmhhc093bih2LGspKTsKICBpZighZXhhY3QodmFsdWUsWyJvYnNlcnZlZCIsImRpYWdub3N0aWMiXSl8fHR5cGVvZiB2YWx1ZS5vYnNlcnZlZCE9PSJib29sZWFuIilyZXR1cm4gbnVsbDsKICBjb25zdCBkPXZhbHVlLmRpYWdub3N0aWM7CiAgaWYoZD09PW51bGwpcmV0dXJuIHtvYnNlcnZlZDp2YWx1ZS5vYnNlcnZlZCxkaWFnbm9zdGljOm51bGx9OwogIGlmKCF2YWx1ZS5vYnNlcnZlZHx8IWV4YWN0KGQsWyJzaXRlIiwiZXhjZXB0aW9uX2NsYXNzIiwib3duX2Z1bmN0aW9uIiwib3duX2xpbmUiXSl8fAogICAgICFbImhhbmRsZXIiLCJzZXJ2ZXIiLCJtYWluIl0uaW5jbHVkZXMoZC5zaXRlKXx8CiAgICAgIVsiQXR0cmlidXRlRXJyb3IiLCJUeXBlRXJyb3IiLCJWYWx1ZUVycm9yIiwiS2V5RXJyb3IiLCJPU0Vycm9yIiwiQnJva2VuUGlwZUVycm9yIiwKICAgICAgICJDb25uZWN0aW9uUmVzZXRFcnJvciIsIlRpbWVvdXRFcnJvciIsIm90aGVyIl0uaW5jbHVkZXMoZC5leGNlcHRpb25fY2xhc3MpKXJldHVybiBudWxsOwogIGNvbnN0IGZ1bmN0aW9ucz1bIkhlYWRlclJlYWRlci5yZWFkbGluZSIsIkRlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0IiwiRGVtby5iZWdpbiIsIkRlbW8uY2FsbGJhY2siLAogICAgIkRlbW8uaW52b2tlIiwiSGFuZGxlci5oYW5kbGVfb25lX3JlcXVlc3QiLCJIYW5kbGVyLnNlbmRfZXJyb3IiLCJIYW5kbGVyLmdldCIsIkhhbmRsZXIucmVwbHkiLCJtYWluIl07CiAgaWYoISgoZC5vd25fZnVuY3Rpb249PT1udWxsJiZkLm93bl9saW5lPT09bnVsbCl8fAogICAgICAgKGZ1bmN0aW9ucy5pbmNsdWRlcyhkLm93bl9mdW5jdGlvbikmJk51bWJlci5pc1NhZmVJbnRlZ2VyKGQub3duX2xpbmUpJiYKICAgICAgICBkLm93bl9saW5lPj0xJiZkLm93bl9saW5lPD0xMDI0KSkpcmV0dXJuIG51bGw7CiAgcmV0dXJuIHtvYnNlcnZlZDp0cnVlLGRpYWdub3N0aWM6e3NpdGU6ZC5zaXRlLGV4Y2VwdGlvbl9jbGFzczpkLmV4Y2VwdGlvbl9jbGFzcywKICAgIG93bl9mdW5jdGlvbjpkLm93bl9mdW5jdGlvbixvd25fbGluZTpkLm93bl9saW5lfX07Cn0KZnVuY3Rpb24gZGlhZ25vc3RpYyh2YWx1ZSxwcm9qZWN0aW9uKSB7CiAgY29uc3QgcHJvamVjdGVkPWZpbml0ZU9ic2VydmF0aW9uKHZhbHVlKTsKICBpZihwcm9qZWN0aW9uPT09ImludmFsaWQifHwhWyJ2YWxpZCIsInVuYXZhaWxhYmxlIl0uaW5jbHVkZXMocHJvamVjdGlvbil8fAogICAgIChwcm9qZWN0aW9uPT09InZhbGlkIiYmcHJvamVjdGVkPT09bnVsbCkpIHsKICAgIGlmKCFyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMuaW5jbHVkZXMoIm9ic2VydmVyX3Byb2plY3Rpb25faW52YWxpZCIpKQogICAgICByZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMucHVzaCgib2JzZXJ2ZXJfcHJvamVjdGlvbl9pbnZhbGlkIik7CiAgfQogIGlmKHByb2plY3RlZCE9PW51bGwmJnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb249PT1udWxsKQogICAgcmVjb3JkLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbj1wcm9qZWN0ZWQ7CiAgcmV0YWluKCk7IC8vIE5vIHJhdyBkaWFnbm9zdGljIGZhbGxiYWNrIGFuZCBubyBmaXJzdC1mYWlsdXJlIG9yIG91dGNvbWUgbXV0YXRpb24uCn0KY29uc3QgbnM9Yz0+QmlnSW50KGMubW9ub3RvbmljX25zKTsKY29uc3QgZ2V0Q2xvY2s9KCk9PmxvY2FsKENMT0NLKTsKbGV0IGNvbnRyb2xsZXJKb2luZWQ9ZmFsc2U7CmxldCBjb250cm9sbGVyUG9sbEF2YWlsYWJsZT10cnVlOwpsZXQgY29udHJvbGxlckJ1ZmZlcj0iIjsKbGV0IGNsZWFudXBTdGFydGVkPWZhbHNlOwpsZXQgbGFzdFNuYXBzaG90RmxhZ3M9bnVsbDsKZnVuY3Rpb24gbGF0Y2gobGFiZWwscmVjZWl2ZWRXYWxsKSB7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICByZWNvcmQuZmlyc3RfZmFpbHVyZT1sYWJlbDsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPXJlY2VpdmVkV2FsbDsKICAgIHJldGFpbigpOyAvLyBTeW5jaHJvbm91cyBmaXJzdC1mYWlsdXJlL3dhbGwgbGF0Y2ggQkVGT1JFIGFueSBuZXcgYXdhaXQvb3V0cHV0LgogIH0KfQpmdW5jdGlvbiBvYnNlcnZlQ29udHJvbGxlcihyLHJlY2VpdmVkV2FsbCkgewogIGNvbnN0IG9ic2VydmF0aW9uPXtyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbCwKICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgIGhlbHBlcl9jb21wbGV0ZWQ6ZmFsc2UsaGVscGVyX2V4aXQ6bnVsbCxmaXh0dXJlX2ZpbmlzaGVkOmZhbHNlfTsKICAvLyBDb21wbGV0ZSBudW1lcmljIHJlc3VsdCBwcm9qZWN0aW9uIGlzIHJldGFpbmVkIEJFRk9SRSBjb21wYXJpc29ucy4KICByZWNvcmQuY29udHJvbGxlcl9vYnNlcnZhdGlvbnMucHVzaChvYnNlcnZhdGlvbik7cmV0YWluKCk7CiAgaWYodHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciIpIHsKICAgIGNvbnRyb2xsZXJKb2luZWQ9dHJ1ZTtyZWNvcmQuY29udHJvbGxlcl9leGl0PXIuZXhpdF9jb2RlO3JldGFpbigpOwogIH0KICBjb250cm9sbGVyQnVmZmVyKz10eXBlb2Ygci5vdXRwdXQ9PT0ic3RyaW5nIj9yLm91dHB1dDoiIjsKICBpZihjb250cm9sbGVyQnVmZmVyLmxlbmd0aD4xNjM4NCkgewogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250cm9sbGVyQnVmZmVyPSIiO3JldHVybjsKICB9CiAgbGV0IGN1dDsKICB3aGlsZSgoY3V0PWNvbnRyb2xsZXJCdWZmZXIuaW5kZXhPZigiXG4iKSk+PTApIHsKICAgIGNvbnN0IGxpbmU9Y29udHJvbGxlckJ1ZmZlci5zbGljZSgwLGN1dCk7Y29udHJvbGxlckJ1ZmZlcj1jb250cm9sbGVyQnVmZmVyLnNsaWNlKGN1dCsxKTsKICAgIGlmKCFsaW5lLnRyaW0oKSljb250aW51ZTsKICAgIGxldCBldmVudDsKICAgIHRyeXtldmVudD1KU09OLnBhcnNlKGxpbmUpO31jYXRjaHsKICAgICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250aW51ZTsKICAgIH0KICAgIGlmKE9iamVjdC5oYXNPd24oZXZlbnQsInVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiIpKQogICAgICBkaWFnbm9zdGljKGV2ZW50LnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbixldmVudC51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoZXZlbnQuZml4dHVyZV9yZWFkeT09PXRydWUpIHsKICAgICAgaWYoZXZlbnQuZ3VhcmRfcGlkIT09b3duLmd1YXJkX3BpZHx8ZXZlbnQuc2VydmVyX3BpZCE9PW93bi5zZXJ2ZXJfcGlkfHwKICAgICAgICAgZXZlbnQubGFiIT09b3duLmxhYnx8IU51bWJlci5pc1NhZmVJbnRlZ2VyKGV2ZW50LmhlbHBlcl9waWQpfHxldmVudC5oZWxwZXJfcGlkPD0wfHwKICAgICAgICAgW293bi5ndWFyZF9waWQsb3duLnNlcnZlcl9waWQsb3duLmJyb3dzZXJfcGlkXS5pbmNsdWRlcyhldmVudC5oZWxwZXJfcGlkKXx8CiAgICAgICAgIChvd24uaGVscGVyX3BpZCE9PW51bGwmJm93bi5oZWxwZXJfcGlkIT09ZXZlbnQuaGVscGVyX3BpZCkpCiAgICAgICAgbGF0Y2goImZpeHR1cmVfcmVhZHlfdW5jb25maXJtZWQiLHJlY2VpdmVkV2FsbCk7CiAgICAgIGVsc2UgewogICAgICAgIG93bi5oZWxwZXJfcGlkPWV2ZW50LmhlbHBlcl9waWQ7b3duLmZpeHR1cmVfcmVhZHk9dHJ1ZTtvd24ubGlzdGVuZXJfcGlkX3Byb29mPXRydWU7CiAgICAgICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICAgICAgfQogICAgfQogICAgaWYoZXZlbnQuaGVscGVyX2NvbXBsZXRlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uaGVscGVyX2NvbXBsZXRlZD10cnVlOwogICAgICBvYnNlcnZhdGlvbi5oZWxwZXJfZXhpdD10eXBlb2YgZXZlbnQuZXhpdD09PSJudW1iZXIiP2V2ZW50LmV4aXQ6bnVsbDsKICAgICAgcmV0YWluKCk7CiAgICAgIGlmKGV2ZW50LmV4aXQhPT0wKWxhdGNoKCJoZWxwZXJfZmFpbGVkIixyZWNlaXZlZFdhbGwpOwogICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSYmCiAgICAgICAgICAgICAgIShvd24ucGhhc2U9PT0iYnJvd3Nlcl9kZWNpc2lvbiImJgogICAgICAgICAgICAgICAgb3duLm5leHRfZGVjaXNpb24/LmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIikpCiAgICAgICAgbGF0Y2goImhlbHBlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50IixyZWNlaXZlZFdhbGwpOwogICAgfQogICAgaWYoZXZlbnQuZml4dHVyZV9maW5pc2hlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uZml4dHVyZV9maW5pc2hlZD10cnVlO3JldGFpbigpOwogICAgICBpZighY2xlYW51cFN0YXJ0ZWQpbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKICAgIH0KICB9CiAgaWYoY29udHJvbGxlckpvaW5lZCYmIWNsZWFudXBTdGFydGVkKQogICAgbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKfQphc3luYyBmdW5jdGlvbiBwb2xsQ29udHJvbGxlcigpIHsKICBpZihjb250cm9sbGVySm9pbmVkfHwhY29udHJvbGxlclBvbGxBdmFpbGFibGUpcmV0dXJuOwogIHRyeSB7CiAgICBjb25zdCByPWF3YWl0IHRvb2xzLndyaXRlX3N0ZGluKHtzZXNzaW9uX2lkOm93bi5leGVjX3Nlc3Npb24sCiAgICAgIGNoYXJzOiIiLHlpZWxkX3RpbWVfbXM6NTAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgICBjb25zdCByZWNlaXZlZFdhbGw9RGF0ZS5ub3coKTsKICAgIG9ic2VydmVDb250cm9sbGVyKHIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHsKICAgIGNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlPWZhbHNlOwogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25fdW5hdmFpbGFibGUiLERhdGUubm93KCkpOwogICAgaWYoY2xlYW51cFN0YXJ0ZWQpY2xlYW51cEVycm9yKCJjb250cm9sbGVyX2pvaW5fdW5jb25maXJtZWQiKTsKICB9Cn0KYXN5bmMgZnVuY3Rpb24gdGltZWREcml2ZXIobGFiZWwsYWxsb2NhdGlvbixvcGVyYXRpb24scHJvamVjdCkgewogIGxldCBzdGFydD1udWxsLGVuZD1udWxsLHJlc3VsdD1udWxsLHN0YXRlPSJ1bmtub3duIjsKICB0cnl7c3RhcnQ9YXdhaXQgZ2V0Q2xvY2soKTt9Y2F0Y2h7Y2xlYW51cEVycm9yKCJjbG9ja191bmF2YWlsYWJsZSIpO30KICBjb25zdCBhY3Rpb249e2xhYmVsLHN0YXJ0X2Nsb2NrOnN0YXJ0LGVuZF9jbG9jazpudWxsLGFsbG93ZWRfbXM6YWxsb2NhdGlvbiwKICAgIHJlc3VsdF9zdGF0ZToidW5rbm93biIsZWxhcHNlZF9tczpudWxsLG92ZXJfYnVkZ2V0Om51bGx9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsgLy8gU3RhcnQgcmV0YWluZWQgQkVGT1JFIGVudGVyaW5nIERyaXZlciBjYWxsLgogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7CiAgICBzdGF0ZT1yZXN1bHQ/LmlzRXJyb3I9PT10cnVlPyJyZWZ1c2VkIjoicmV0dXJuZWQiOwogIH0gY2F0Y2gge3N0YXRlPSJleGNlcHRpb24iO30KICB0cnl7ZW5kPWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgYWN0aW9uLmVuZF9jbG9jaz1lbmQ7YWN0aW9uLnJlc3VsdF9zdGF0ZT1zdGF0ZTsKICByZXRhaW4oKTsgLy8gRW5kL3Jlc3VsdCByZXRhaW5lZCBCRUZPUkUgYnVkZ2V0IGNvbXBhcmlzb24uCiAgaWYoc3RhcnQmJmVuZCkgewogICAgY29uc3QgZD1ucyhlbmQpLW5zKHN0YXJ0KTsKICAgIGlmKGQ+PTBuKSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcihkLzEwMDAwMDBuKTsKICAgICAgYWN0aW9uLm92ZXJfYnVkZ2V0PWFjdGlvbi5lbGFwc2VkX21zPmFsbG9jYXRpb247CiAgICB9IGVsc2UgY2xlYW51cEVycm9yKCJjbG9ja19pbnZhbGlkIik7CiAgfQogIGlmKGFjdGlvbi5vdmVyX2J1ZGdldCljbGVhbnVwRXJyb3IoImRyaXZlcl9vcGVyYXRpb25fb3Zlcl9idWRnZXQiKTsKICBpZihzdGF0ZSE9PSJyZXR1cm5lZCIpY2xlYW51cEVycm9yKCJkcml2ZXJfb3BlcmF0aW9uX3VuY29uZmlybWVkIik7CiAgaWYocHJvamVjdClwcm9qZWN0KHJlc3VsdCxzdGF0ZSk7CiAgcmV0YWluKCk7Cn0KYXN5bmMgZnVuY3Rpb24gY2xlYW51cCgpIHsKICBzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsgLy8gQ2xlYXIgYmVmb3JlIGFueSBjbGVhbnVwIGF3YWl0LgogIGlmKGNsZWFudXBTdGFydGVkKXJldHVybjsKICBjbGVhbnVwU3RhcnRlZD10cnVlO3JlY29yZC5jbGVhbnVwX3N0YXJ0ZWQ9dHJ1ZTtyZXRhaW4oKTsKICB0cnkgewogICAgY29uc3Qgcj1hd2FpdCBsb2NhbChTVE9QLHsKICAgICAgbGFiOm93bi5sYWIsZXZlbnRfb3V0Om93bi5ldmVudF9vdXQsCiAgICAgIGZpcnN0X2ZhaWx1cmU6cmVjb3JkLmZpcnN0X2ZhaWx1cmUsCiAgICAgIGZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXM6cmVjb3JkLmZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXMKICAgIH0pOwogICAgcmVjb3JkLmZpcnN0X2Nsb2NrPXIuY2xvY2s7cmVjb3JkLnN0b3Bfc3RhdGU9ci5tYXJrZXJfc3RhdGU7cmV0YWluKCk7CiAgICBpZihyLmV2ZW50X3N0YXRlIT09IndyaXR0ZW4iKWNsZWFudXBFcnJvcigib2JzZXJ2YXRpb25fbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICAgIGlmKHIubWFya2VyX3N0YXRlPT09InN0b3BfdW5jb25maXJtZWQiKWNsZWFudXBFcnJvcigic3RvcF91bmNvbmZpcm1lZCIpOwogICAgaWYoci5tYXJrZXJfc3RhdGU9PT0ib3duZXJzaGlwX3Vua25vd24iKWNsZWFudXBFcnJvcigib3duZXJzaGlwX3Vua25vd24iKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoInN0b3Bfb3JfY2xvY2tfcmVjb3JkX3VuYXZhaWxhYmxlIik7fQogIC8vIE5vIG91dHN0YW5kaW5nIERyaXZlciBjYWxsIGV4aXN0cyBoZXJlOiBhbGwgcGFnZSBjYWxscyBhYm92ZSB3ZXJlIGF3YWl0ZWQuCiAgLy8gT3JpZ2luYWwgc3RvcCBwcm90b2NvbCBhbHJlYWR5IGNhdXNlcyB0aGUgY29udHJvbGxlcidzIG93bmVkLWNoaWxkIGZpbmFsbHkuCiAgYXdhaXQgdGltZWREcml2ZXIoImtpbGxfYXBwIiwzMDAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2tpbGxfYXBwKHtwaWQ6b3duLmJyb3dzZXJfcGlkfSkpOwogIGF3YWl0IHRpbWVkRHJpdmVyKCJlbmRfc2Vzc2lvbiIsMTUwMDAsCiAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19lbmRfc2Vzc2lvbih7c2Vzc2lvbjpvd24uc2Vzc2lvbn0pLChyLHN0YXRlKT0+ewogICAgICByZWNvcmQuc2Vzc2lvbl9lbmRlZD1zdGF0ZT09PSJyZXR1cm5lZCImJgogICAgICAgIHI/LnN0cnVjdHVyZWRDb250ZW50Py5hY3RpdmU9PT1mYWxzZSYmci5zdHJ1Y3R1cmVkQ29udGVudC5zZXNzaW9uPT09b3duLnNlc3Npb247CiAgICAgIHJldGFpbigpOwogICAgfSk7CiAgYXdhaXQgdGltZWREcml2ZXIoImxpc3Rfd2luZG93cyIsNTAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2xpc3Rfd2luZG93cyh7cGlkOm93bi5icm93c2VyX3BpZH0pLChyLHN0YXRlKT0+ewogICAgICBjb25zdCB3aW5kb3dzPXI/LnN0cnVjdHVyZWRDb250ZW50Py53aW5kb3dzOwogICAgICByZWNvcmQud2luZG93X2NvdW50PXN0YXRlPT09InJldHVybmVkIiYmQXJyYXkuaXNBcnJheSh3aW5kb3dzKT93aW5kb3dzLmxlbmd0aDpudWxsOwogICAgICByZXRhaW4oKTsKICAgIH0pOwogIGxldCByZWFkU3RhcnQ9bnVsbDsKICB0cnl7cmVhZFN0YXJ0PWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgY29uc3QgYWN0aW9uPXtsYWJlbDoib3duZWRfam9pbl9yZWFkYmFjayIsc3RhcnRfY2xvY2s6cmVhZFN0YXJ0LGVuZF9jbG9jazpudWxsLAogICAgYWxsb3dlZF9tczpudWxsLGVsYXBzZWRfbXM6bnVsbCxvdmVyX2J1ZGdldDpudWxsLHJlc3VsdF9zdGF0ZToidW5rbm93biJ9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsKICBpZihyZWFkU3RhcnQmJnJlY29yZC5maXJzdF9jbG9jaykgewogICAgYWN0aW9uLmFsbG93ZWRfbXM9TWF0aC5tYXgoMCw2MDAwMC1OdW1iZXIoKG5zKHJlYWRTdGFydCktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pKTsKICAgIHJldGFpbigpOwogIH0KICAvLyBDb250aW51ZSBlc3NlbnRpYWwgam9pbiBhZnRlcjYwcyBpZiBsYXRlOyBuZXZlciBjbGFpbSB0aGF0IGRlYWRsaW5lIGVuZm9yY2VkLgogIC8vIERvIG5vdCBwb2xsIGFub3RoZXIgZXhlYyBzZXNzaW9uIG9yIHNlbmQgYSBtYW51YWwgcHJvY2VzcyBzaWduYWwuCiAgd2hpbGUoIWNvbnRyb2xsZXJKb2luZWQmJmNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlJiZEYXRlLm5vdygpPG93bi5zdGFydF9lcG9jaF9tcys5MDAwMDApIHsKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spIHsKICAgICAgdHJ5IHsKICAgICAgICBjb25zdCBjPWF3YWl0IGdldENsb2NrKCk7cmVjb3JkLmxhc3Rfam9pbl9jbG9jaz1jO3JldGFpbigpOwogICAgICAgIGlmKG5zKGMpLW5zKHJlY29yZC5maXJzdF9jbG9jayk+NjAwMDAwMDAwMDBuKQogICAgICAgICAgY2xlYW51cEVycm9yKCJjbGVhbnVwX2J1ZGdldF9leGNlZWRlZCIpOwogICAgICB9IGNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgICB9CiAgfQogIGlmKCFjb250cm9sbGVySm9pbmVkKWNsZWFudXBFcnJvcigiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIik7CiAgdHJ5IHsKICAgIGNvbnN0IHI9YXdhaXQgbG9jYWwoUkVBREJBQ0ssewogICAgICBvd25lZF9waWRzOltvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCwKICAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSwKICAgICAgbGFiOm93bi5sYWIsb3V0ZXJfb3V0Om93bi5vdXRlcl9vdXQsaGVscGVyX3BpZDpvd24uaGVscGVyX3BpZCxzZXJ2ZXJfcGlkOm93bi5zZXJ2ZXJfcGlkCiAgICB9KTsKICAgIGFjdGlvbi5lbmRfY2xvY2s9ci5jbG9jazthY3Rpb24ucmVzdWx0X3N0YXRlPSJyZXR1cm5lZCI7CiAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHM9ci5vd25lZF9jaGlsZF9leGl0czsKICAgIGRpYWdub3N0aWMoci51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sci51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoTnVtYmVyLmlzU2FmZUludGVnZXIoci5oZWxwZXJfcGlkKSYmci5oZWxwZXJfcGlkPjApb3duLmhlbHBlcl9waWQ9ci5oZWxwZXJfcGlkOwogICAgcmVjb3JkLmZpbmFsX2Fic2VuY2U9e293bmVkX3BpZHM6ci5vd25lZF9waWRzLHBvcnRzOnIucG9ydHMsCiAgICAgIGxhYjpyLmxhYl9hYnNlbnQsd2luZG93X2NvdW50OnJlY29yZC53aW5kb3dfY291bnQ/P251bGwsCiAgICAgIHNlc3Npb25fZW5kZWQ6cmVjb3JkLnNlc3Npb25fZW5kZWQ9PT10cnVlfTsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zPXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPT09bnVsbD9udWxsOgogICAgICBEYXRlLm5vdygpLXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOwogICAgcmV0YWluKCk7IC8vIEZ1bGwgZml4ZWQgcmVhZGJhY2svZXhpdHMvY2xvY2sgQkVGT1JFIGNvbXBhcmlzb25zLgogICAgaWYocmVhZFN0YXJ0KSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVhZFN0YXJ0KSkvMTAwMDAwMG4pOwogICAgICBhY3Rpb24ub3Zlcl9idWRnZXQ9YWN0aW9uLmFsbG93ZWRfbXM9PT1udWxsP251bGw6YWN0aW9uLmVsYXBzZWRfbXM+YWN0aW9uLmFsbG93ZWRfbXM7CiAgICB9CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spCiAgICAgIHJlY29yZC5sYXRjaF90b19hYnNlbmNlX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pOwogICAgaWYoYWN0aW9uLm92ZXJfYnVkZ2V0KWNsZWFudXBFcnJvcigiY2xlYW51cF9idWRnZXRfZXhjZWVkZWQiKTsKICAgIGNvbnN0IGE9cmVjb3JkLmZpbmFsX2Fic2VuY2U7CiAgICBpZighY29udHJvbGxlckpvaW5lZHx8IUFycmF5LmlzQXJyYXkocmVjb3JkLm93bmVkX2NoaWxkX2V4aXRzKXx8CiAgICAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHMubGVuZ3RoIT09KG93bi5oZWxwZXJfcGlkPT09bnVsbD82OjcpKQogICAgICBjbGVhbnVwRXJyb3IoImNoaWxkX3JlYXBfaW5jb21wbGV0ZSIpOwogICAgaWYoIU9iamVjdC52YWx1ZXMoYS5vd25lZF9waWRzKS5ldmVyeSh2PT52PT09dHJ1ZSl8fAogICAgICAgIU9iamVjdC52YWx1ZXMoYS5wb3J0cykuZXZlcnkodj0+dj09PXRydWUpfHxhLmxhYiE9PXRydWV8fAogICAgICAgYS53aW5kb3dfY291bnQhPT0wfHxhLnNlc3Npb25fZW5kZWQhPT10cnVlKQogICAgICBjbGVhbnVwRXJyb3IoIm93bmVkX3Jlc291cmNlX2Fic2VuY2VfdW5wcm92ZW4iKTsKICB9IGNhdGNoIHthY3Rpb24ucmVzdWx0X3N0YXRlPSJleGNlcHRpb24iO2NsZWFudXBFcnJvcigib3duZWRfcmVhZGJhY2tfdW5hdmFpbGFibGUiKTt9CiAgY2xlYW51cEVycm9yKCJmaXJzdF9ldmVudF91bnByb3ZlbiIpOwogIHJlY29yZC53aG9sZV9jbGVhbnVwX3dpdGhpbjYwX3Byb3Zlbj1mYWxzZTtyZXRhaW4oKTsKICAvLyBFeGNsdXNpdmUgbmV3IGZpbGUgb25seTsgZXhpc3Rpbmcgb3V0ZXIvaGVscGVyL3Byb3ZpZGVyIGV2aWRlbmNlIHVudG91Y2hlZC4KICB0cnkgewogICAgY29uc3QgY21kPSJweXRob24zIC1jICIrc3EoUEVSU0lTVCkrIiAiK3NxKG93bi5jbGVhbnVwX291dCkrIiAiK3NxKEpTT04uc3RyaW5naWZ5KHJlY29yZCkpOwogICAgY29uc3Qgcj1hd2FpdCB0b29scy5leGVjX2NvbW1hbmQoe2NtZCx3b3JrZGlyOm93bi53b3Jrc3BhY2UsCiAgICAgIHlpZWxkX3RpbWVfbXM6MTAwMDAsbWF4X291dHB1dF90b2tlbnM6NTAwfSk7CiAgICByZWNvcmQubG9jYWxfdG9vbF9yZWNlaXB0cy5wdXNoKHtsYWJlbDoicGVyc2lzdCIsCiAgICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGx9KTtyZXRhaW4oKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZHx8ci5leGl0X2NvZGUhPT0wKQogICAgICBjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTt9CiAgY29udHJvbGxlckJ1ZmZlcj0iIjtvd24uY2xvc2VkPXRydWU7c3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKfQphc3luYyBmdW5jdGlvbiBjaGVja2VkKGxhYmVsLG9wZXJhdGlvbixwcmVkaWNhdGUpIHsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpcmV0dXJuIG51bGw7CiAgaWYoRGF0ZS5ub3coKT49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkgewogICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuIG51bGw7CiAgfQogIGxldCByZXN1bHQscmVjZWl2ZWRXYWxsOwogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7cmVjZWl2ZWRXYWxsPURhdGUubm93KCk7CiAgICByZWNvcmQub2JzZXJ2YXRpb25fcmVjZWlwdHMucHVzaCh7a2luZDpsYWJlbCxyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbH0pO3JldGFpbigpOwogICAgaWYocmVzdWx0Py5pc0Vycm9yPT09dHJ1ZSlsYXRjaCgiYnJvd3Nlcl90b29sX3JlZnVzZWQiLHJlY2VpdmVkV2FsbCk7CiAgICBlbHNlIGlmKCFwcmVkaWNhdGUocmVzdWx0KSlsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHtsYXRjaCgiYnJvd3Nlcl90b29sX29yX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7fQogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVhbnVwKCk7CiAgcmV0dXJuIHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbD9yZXN1bHQ6bnVsbDsKfQpjb25zdCBzbmFwc2hvdEFyZ3M9e3Nlc3Npb246b3duLnNlc3Npb24sdGFyZ2V0X2lkOm93bi50YXJnZXRfaWQsdGFiX2lkOm93bi50YWJfaWQsCiAgc25hcHNob3RfZm9ybWF0OiJzZW1hbnRpY192MiIsaW5jbHVkZV9zY3JlZW5zaG90OmZhbHNlfTsKZnVuY3Rpb24gcGFnZShyZXN1bHQsa2luZCkgewogIGNvbnN0IHM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudCxub2Rlcz1BcnJheS5pc0FycmF5KHM/LmNvbnRlbnRfcmVmcyk/cy5jb250ZW50X3JlZnM6W107CiAgY29uc3QgZmxhZ3M9ewogICAgc3RhdHVzX29rOnJlc3VsdD8uaXNFcnJvciE9PXRydWUmJnM/LnN0YXR1cz09PSJvayIsCiAgICBjb21wbGV0ZTpzPy5zbmFwc2hvdD8uY29tcGxldGU9PT10cnVlLAogICAgaGVhZGluZ19tYXRjaDpub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09CiAgICAgIChraW5kPT09InByb3RlY3RlZF9hZnRlciI/IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiOiJMb2NhbCBkZW1vIikpLAogICAgZXJyb3JfbWF0Y2g6bm9kZXMuc29tZShuPT5uLm5hbWU9PT0iTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LiIpLAogICAgcmVxdWlyZWRfdGV4dDpub2Rlcy5zb21lKG49Pm4ubmFtZT09PShraW5kPT09InByb3RlY3RlZF9iZWZvcmUiPyJTaWduIGluIHJlcXVpcmVkLiI6CiAgICAgIGtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIj8iU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iOgogICAgICAiVXNlIFNpZ24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLiIpKSwKICAgIGludGVyYWN0aXZlX3JlZl9wcmVzZW50OkFycmF5LmlzQXJyYXkocz8ucmVmcykmJgogICAgICBzLnJlZnMuc29tZShuPT5BcnJheS5pc0FycmF5KG4uYWN0aW9ucykmJm4uYWN0aW9ucy5pbmNsdWRlcygiY2xpY2siKSkKICB9OwogIHJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M9e2tpbmQsLi4uZmxhZ3N9O3JldGFpbigpOwogIC8vIE9ubHkgcHJvdmlkZXIgcmVmcyBhbmQgZml4ZWQgcHVibGljLWxhYmVsIG1hdGNoZXMgc3Vydml2ZSB0aGlzIHJhdyBzbmFwc2hvdC4KICBvd24uZnJlc2hfcmVmcz1BcnJheS5pc0FycmF5KHM/LnJlZnMpP3MucmVmcy5maWx0ZXIobj0+dHlwZW9mIG4ucmVmPT09InN0cmluZyImJgogICAgL15wWzAtOV0rOlswLTldKyQvLnRlc3Qobi5yZWYpKS5tYXAobj0+bi5yZWYpOltdOwogIHN0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7CiAgaWYoZmxhZ3MuZXJyb3JfbWF0Y2gpcmV0dXJuIGZhbHNlOwogIGlmKCFmbGFncy5zdGF0dXNfb2t8fCFmbGFncy5jb21wbGV0ZSlyZXR1cm4gZmFsc2U7CiAgaWYoa2luZD09PSJnZW5lcmljX2NoZWNrZWQiKSB7CiAgICBpZihub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiKSYmCiAgICAgICBub2Rlcy5zb21lKG49Pm4ubmFtZT09PSJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZhaWxhYmxlLiIpKQogICAgICByZWNvcmQucHJvdGVjdGVkX2FmdGVyX3BhZ2U9dHJ1ZTsKICAgIHJldHVybiB0cnVlOwogIH0KICBjb25zdCBvaz1mbGFncy5oZWFkaW5nX21hdGNoJiZmbGFncy5yZXF1aXJlZF90ZXh0JiYKICAgIChraW5kIT09ImFwcGxpY2F0aW9uInx8ZmxhZ3MuaW50ZXJhY3RpdmVfcmVmX3ByZXNlbnQpOwogIGlmKG9rJiZraW5kPT09InByb3RlY3RlZF9hZnRlciIpcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlPXRydWU7CiAgcmV0dXJuIG9rOwp9CmFzeW5jIGZ1bmN0aW9uIG5hdmlnYXRlQW5kU25hcHNob3QodXJsLGtpbmQpIHsKICBpZihhd2FpdCBjaGVja2VkKCJicm93c2VyX25hdmlnYXRpb24iLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19icm93c2VyX25hdmlnYXRlKHtzZXNzaW9uOm93bi5zZXNzaW9uLAogICAgICAgIHRhcmdldF9pZDpvd24udGFyZ2V0X2lkLHRhYl9pZDpvd24udGFiX2lkLHVybH0pLAogICAgICByPT5yPy5pc0Vycm9yIT09dHJ1ZSkpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLHI9PnBhZ2UocixraW5kKSk7CiAgfQp9CmFzeW5jIGZ1bmN0aW9uIGVudHJ5KCkgewogIC8vIFRoZSBleGFjdCBEcml2ZXItb3duZWQgYmxhbmsgaGFuZGxlcyBhbHJlYWR5IGV4aXN0IHdoaWxlIHRoZTE4MHMgZ2F0ZSB3YWl0cy4KICAvLyBUaGVyZSBpcyBubyBtb2RlbCB5aWVsZCwgdG9vbC1kZXNjcmlwdGlvbiBsb2FkIG9yIHJlYmluZCBhZnRlciB0aGlzIG1hcmtlci4KICBjb25zdCBtYXJrZXJDb21tYW5kPSJweXRob24zIC1jICIrc3EoTUFSS0VSKSsiICIrc3Eob3duLmxhYikrIiAiKwogICAgc3Eob3duLmd1YXJkX3BpZCkrIiAiK3NxKG93bi5zZXJ2ZXJfcGlkKTsKICBhd2FpdCBjaGVja2VkKCJwcmVwYXJlZF9tYXJrZXIiLAogICAgKCk9PnRvb2xzLmV4ZWNfY29tbWFuZCh7Y21kOm1hcmtlckNvbW1hbmQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgICB5aWVsZF90aW1lX21zOjEwMDAwLG1heF9vdXRwdXRfdG9rZW5zOjUwMH0pLHI9PnsKICAgICAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7bGFiZWw6Im1hcmtlciIsCiAgICAgICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgICAgIHNlc3Npb25faWQ6dHlwZW9mIHIuc2Vzc2lvbl9pZD09PSJudW1iZXIiP3Iuc2Vzc2lvbl9pZDpudWxsfSk7cmV0YWluKCk7CiAgICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgICAgcmV0dXJuIHIuZXhpdF9jb2RlPT09MCYmci5zZXNzaW9uX2lkPT09dW5kZWZpbmVkJiYKICAgICAgICBKU09OLnBhcnNlKHIub3V0cHV0KS5wcmVwYXJlZF9tYXJrZXJfd3JpdHRlbj09PXRydWU7CiAgICB9KTsKICBjb25zdCByZWFkeURlYWRsaW5lPURhdGUubm93KCkrMzAwMDA7CiAgd2hpbGUocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUmJkRhdGUubm93KCk8cmVhZHlEZWFkbGluZSkKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUpCiAgICBsYXRjaCgiZml4dHVyZV9yZWFkeV91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOyAvLyBPTkUgaW1tZWRpYXRlIHByZS1uYXZpZ2F0aW9uIHBvbGwsIG5vIFJQIEhUVFAgcHJvYmUuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBuYXZpZ2F0ZUFuZFNuYXBzaG90KCJodHRwOi8vbG9jYWxob3N0OjMwMDAvcHJvdGVjdGVkIiwicHJvdGVjdGVkX2JlZm9yZSIpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbClhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgYXdhaXQgbmF2aWdhdGVBbmRTbmFwc2hvdCgiaHR0cDovL2xvY2FsaG9zdDozMDAwLyIsImFwcGxpY2F0aW9uIik7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIE9ORSBwb3N0LWVudHJ5IHBvbGwuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICBvd24ucGhhc2U9ImJyb3dzZXJfZGVjaXNpb24iOwogICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfQphc3luYyBmdW5jdGlvbiBjb250aW51YXRpb24oKSB7CiAgY29uc3Qgc3BlYz1vd24ubmV4dF9kZWNpc2lvbjsKICBjb25zdCBhbGxvd2VkS2luZHM9WyJjbGljayIsInVzZXJuYW1lIiwicGFzc3dvcmQiLCJzbmFwc2hvdCIsInByb3RlY3RlZF9hZnRlciJdOwogIGlmKCFzcGVjfHwhYWxsb3dlZEtpbmRzLmluY2x1ZGVzKHNwZWMua2luZCkpIHsKICAgIGxhdGNoKCJicm93c2VyX2RlY2lzaW9uX3VuY29uZmlybWVkIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuOwogIH0KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgaWYoc3BlYy5raW5kPT09InByb3RlY3RlZF9hZnRlciIpIHsKICAgIC8vIFJlYWQgdGhlIGZyZXNoIGNhbGxiYWNrLWZvbGxvd2luZyBwcm90ZWN0ZWQgcGFnZTsgZG8gbm90IHNlbmQgYW5vdGhlciBSUCByZXF1ZXN0LgogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncykscj0+cGFnZShyLCJwcm90ZWN0ZWRfYWZ0ZXIiKSk7CiAgfSBlbHNlIGlmKHNwZWMua2luZD09PSJzbmFwc2hvdCIpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLAogICAgICByPT5wYWdlKHIsImdlbmVyaWNfY2hlY2tlZCIpJiZwdWJsaWNQcmVkaWNhdGUocixzcGVjKSk7CiAgfSBlbHNlIHsKICAgIGlmKHR5cGVvZiBzcGVjLnJlZiE9PSJzdHJpbmcifHwhQXJyYXkuaXNBcnJheShvd24uZnJlc2hfcmVmcyl8fAogICAgICAgIW93bi5mcmVzaF9yZWZzLmluY2x1ZGVzKHNwZWMucmVmKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIG93bi5mcmVzaF9yZWZzPVtdO3N0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7IC8vIFNpbmdsZS11c2Ugc25hcHNob3QgcmVmLgogICAgY29uc3QgYXJncz17c2Vzc2lvbjpvd24uc2Vzc2lvbix0YXJnZXRfaWQ6b3duLnRhcmdldF9pZCx0YWJfaWQ6b3duLnRhYl9pZCxyZWY6c3BlYy5yZWZ9OwogICAgY29uc3Qgb3BlcmF0aW9uPXNwZWMua2luZD09PSJjbGljayI/CiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2Jyb3dzZXJfY2xpY2soey4uLmFyZ3MsaW5wdXRfcm91dGU6ImRvbV9ldmVudCJ9KToKICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fYnJvd3Nlcl90eXBlKHsuLi5hcmdzLHJlcGxhY2U6dHJ1ZSwKICAgICAgICB0ZXh0OnNwZWMua2luZD09PSJ1c2VybmFtZSI/ImFkbWluIjpsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKX0pOwogICAgaWYoc3BlYy5raW5kPT09InBhc3N3b3JkIiYmKHR5cGVvZiBsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKSE9PSJzdHJpbmcifHwKICAgICAgICEvXltBLVphLXowLTlfLV17MzAsMTI4fSQvLnRlc3QobG9hZCgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IikpKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfaW5wdXQiLG9wZXJhdGlvbixyPT57CiAgICAgIGNvbnN0IHM9cj8uc3RydWN0dXJlZENvbnRlbnQ7CiAgICAgIHJldHVybiByPy5pc0Vycm9yIT09dHJ1ZSYmWyJjb25maXJtZWQiLCJ1bnZlcmlmaWFibGUiXS5pbmNsdWRlcyhzPy5lZmZlY3QpOwogICAgfSk7IC8vIERpc3BhdGNoIGFsb25lIG5ldmVyIGVhcm5zIGFuIGFwcGxpY2F0aW9uIG91dGNvbWUgb3Igam91cm5leSBjcmVkaXQuCiAgICBpZihzcGVjLmtpbmQ9PT0icGFzc3dvcmQiKXN0b3JlKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiLG51bGwpOwogICAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKQogICAgICBhd2FpdCBjaGVja2VkKCJicm93c2VyX3NuYXBzaG90IiwKICAgICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLAogICAgICAgIHI9PnBhZ2UociwiZ2VuZXJpY19jaGVja2VkIikmJnB1YmxpY1ByZWRpY2F0ZShyLHNwZWMpKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKICBlbHNlIGlmKHNwZWMua2luZD09PSJwcm90ZWN0ZWRfYWZ0ZXIiKSB7CiAgICByZWNvcmQuY2xlYW51cF9yZXF1ZXN0ZWRfd2FsbF9tcz1EYXRlLm5vdygpO3JldGFpbigpOwogICAgYXdhaXQgY2xlYW51cCgpOwogIH0KfQpmdW5jdGlvbiBwdWJsaWNQcmVkaWNhdGUocmVzdWx0LHNwZWMpIHsKICBjb25zdCBub2Rlcz1yZXN1bHQ/LnN0cnVjdHVyZWRDb250ZW50Py5jb250ZW50X3JlZnM7CiAgLy8gUm9vdCBtdXN0IHBpbiBvbmUgcHJpbnRlZCBwdWJsaWMgZXhwZWN0ZWQgbGFiZWwvcm9sZSBiZWZvcmUgdGhhdCBhY3Rpb24uCiAgLy8gTm8gcmF3IGZpZWxkIHZhbHVlLCBVUkwsIHN1YmplY3QsIHF1ZXJ5IG9yIGFyYml0cmFyeSByZWdleCBwcmVkaWNhdGUgaXMgYWxsb3dlZC4KICBjb25zdCByb2xlcz1bImhlYWRpbmciLCJidXR0b24iLCJzdGF0aWN0ZXh0IiwidGV4dGJveCJdOwogIGNvbnN0IGxhYmVscz1bIlVzZXJuYW1lIiwiUGFzc3dvcmQiLCJBdXRoZW50aWNhdG9yIG9yIHJlY292ZXJ5IGNvZGUiLCJTaWduIGluIiwKICAgICJMb2NhbCBkZW1vIiwiUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyIsCiAgICAiU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iXTsKICByZXR1cm4gQXJyYXkuaXNBcnJheShub2RlcykmJnJvbGVzLmluY2x1ZGVzKHNwZWMuZXhwZWN0ZWRfcm9sZSkmJgogICAgbGFiZWxzLmluY2x1ZGVzKHNwZWMuZXhwZWN0ZWRfbGFiZWwpJiYKICAgIG5vZGVzLnNvbWUobj0+bi5yb2xlPT09c3BlYy5leHBlY3RlZF9yb2xlJiZuLm5hbWU9PT1zcGVjLmV4cGVjdGVkX2xhYmVsKTsKfQp0cnkgewogIGlmKG93bi5waGFzZT09PSJwcmVwYXJlZF9lbnRyeSIpYXdhaXQgZW50cnkoKTtlbHNlIGF3YWl0IGNvbnRpbnVhdGlvbigpOwogIC8vIERvIG5vdCBjYXJyeSB1bnZhbGlkYXRlZCBwYXJ0aWFsIGNvbnRyb2xsZXIgdGV4dCBhY3Jvc3MgdGhpcyBjZWxsIGJvdW5kYXJ5LgogIHdoaWxlKGNvbnRyb2xsZXJCdWZmZXIubGVuZ3RoPjAmJnJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbCkgewogICAgY29uc3QgYmVmb3JlUG9sbFdhbGw9RGF0ZS5ub3coKTsKICAgIGlmKGJlZm9yZVBvbGxXYWxsPj1vd24uc3RhcnRfZXBvY2hfbXMrODQwMDAwKSB7CiAgICAgIGxhdGNoKCJicm93c2VyX2FjdGl2ZV9kZWFkbGluZSIsYmVmb3JlUG9sbFdhbGwpO2JyZWFrOwogICAgfQogICAgaWYoY29udHJvbGxlckpvaW5lZHx8IWNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlKSB7CiAgICAgIGxhdGNoKCJjb250cm9sbGVyX29ic2VydmF0aW9uX2ludmFsaWQiLGJlZm9yZVBvbGxXYWxsKTticmVhazsKICAgIH0KICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIFNhbWUgb3duZWQgc2Vzc2lvbjsgb2JzZXJ2ZXIgcmV0YWlucyByZWNlaXB0IGZpcnN0LgogICAgY29uc3QgYWZ0ZXJQb2xsV2FsbD1EYXRlLm5vdygpOwogICAgaWYoYWZ0ZXJQb2xsV2FsbD49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkKICAgICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixhZnRlclBvbGxXYWxsKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfSBjYXRjaCB7CiAgbGF0Y2goImNvbXBvc2VkX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7CiAgYXdhaXQgY2xlYW51cCgpOwp9CmlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCkgewogIHRleHQoe3Jlc3VsdDoiZmFpbGVkIixmaXJzdF9mYWlsdXJlOnJlY29yZC5maXJzdF9mYWlsdXJlLAogICAgdW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uOnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sCiAgICBkaWFnbm9zdGljX2Vycm9yczpyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMsY2xlYW51cF9lcnJvcnM6cmVjb3JkLmNsZWFudXBfZXJyb3JzLAogICAgZmluYWxfYWJzZW5jZTpyZWNvcmQuZmluYWxfYWJzZW5jZSxjb250cm9sbGVyX2V4aXQ6cmVjb3JkLmNvbnRyb2xsZXJfZXhpdCwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogICAgcmVzb3VyY2VfcmVsZWFzZV9wcm92ZW46cmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKX0pOwp9IGVsc2UgewogIHRleHQoe3Jlc3VsdDoiYnJvd3Nlcl9kZWNpc2lvbl9vYnNlcnZlZF9vbmx5IixwYWdlX2ZsYWdzOnJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M/P251bGwsCiAgICBqb3VybmV5X2NyZWRpdDpmYWxzZSxjbGVhbnVwX2Vycm9yczpyZWNvcmQuY2xlYW51cF9lcnJvcnMsCiAgICByZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpyZWNvcmQuY2xlYW51cF9zdGFydGVkPT09dHJ1ZSYmcmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKSwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlfSk7Cn0K",
  "diff_b64": "LS0tIGEvYXJjaGl2ZWQtZDAxLWNvbnRpbnVhdGlvbi1jZWxsLmpzCisrKyBiL2FyY2hpdmVkLWQwMS1jb250aW51YXRpb24tY2VsbC5qcwpAQCAtMzI4LDAgKzMyOSBAQAorICBvd24ucHVibGljX2RlY2lzaW9uPXByb2plY3RQdWJsaWNEZWNpc2lvbihyZXN1bHQpOwpAQCAtNDMzLDAgKzQzNSwyNyBAQAorZnVuY3Rpb24gcHJvamVjdFB1YmxpY0RlY2lzaW9uKHJlc3VsdCkgeworICBjb25zdCBzPXJlc3VsdD8uc3RydWN0dXJlZENvbnRlbnQsbm9kZXM9QXJyYXkuaXNBcnJheShzPy5jb250ZW50X3JlZnMpP3MuY29udGVudF9yZWZzOltdOworICBjb25zdCBsYWJlbHM9WworICAgIFsiaGVhZGluZyIsIkxvY2FsIGRlbW8iXSxbImhlYWRpbmciLCJTaWduIGluIHRvIGNvbnRpbnVlIHRvIExvY2FsIGRlbW8iXSwKKyAgICBbImhlYWRpbmciLCJMb2NhbCBkZW1vIHdhbnRzIHRvIHVzZSB5b3VyIHJpQXV0aCBhY2NvdW50Il0sCisgICAgWyJoZWFkaW5nIiwiQ29udGludWUgdG8gTG9jYWwgZGVtbz8iXSxbImhlYWRpbmciLCJQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIl0sCisgICAgWyJidXR0b24iLCJTaWduIGluIl0sWyJidXR0b24iLCJBbGxvdyJdLFsiYnV0dG9uIiwiQ29udGludWUiXSwKKyAgICBbInRleHRib3giLCJVc2VybmFtZSJdLFsidGV4dGJveCIsIlBhc3N3b3JkIl0sCisgICAgWyJ0ZXh0Ym94IiwiQXV0aGVudGljYXRvciBvciByZWNvdmVyeSBjb2RlIChpZiBlbmFibGVkKSJdLAorICAgIFsic3RhdGljdGV4dCIsIlNpZ25lZCBpbi4gUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyBpcyBhdmFpbGFibGUuIl0KKyAgXTsKKyAgY29uc3QgdmFsaWQ9cmVzdWx0Py5pc0Vycm9yIT09dHJ1ZSYmcz8uc3RhdHVzPT09Im9rIiYmcz8uc25hcHNob3Q/LmNvbXBsZXRlPT09dHJ1ZSYmCisgICAgIW5vZGVzLnNvbWUobj0+bi5uYW1lPT09IkxvY2FsIGRlbW8gY291bGQgbm90IGNvbXBsZXRlIHRoaXMgcmVxdWVzdC4iKTsKKyAgcmV0dXJuIHsKKyAgICBvYnNlcnZlZDp2YWxpZD9sYWJlbHMuZmlsdGVyKChbcm9sZSxuYW1lXSk9PgorICAgICAgbm9kZXMuc29tZShuPT5uLnJvbGU9PT1yb2xlJiZuLm5hbWU9PT1uYW1lKSkubWFwKChbcm9sZSxuYW1lXSk9Pih7cm9sZSxuYW1lfSkpOltdLAorICAgIHJlZnM6dmFsaWQmJkFycmF5LmlzQXJyYXkocz8ucmVmcyk/cy5yZWZzLmZsYXRNYXAobj0+eworICAgICAgY29uc3QgcGFpcj1sYWJlbHMuZmluZCgoW3JvbGUsbmFtZV0pPT5uPy5yb2xlPT09cm9sZSYmbi5uYW1lPT09bmFtZSkscmVmPW4/LnJlZjsKKyAgICAgIGlmKCFwYWlyfHx0eXBlb2YgcmVmIT09InN0cmluZyJ8fCEvXnBbMC05XSs6WzAtOV0rJC8udGVzdChyZWYpKXJldHVybiBbXTsKKyAgICAgIGNvbnN0IFtyb2xlLG5hbWVdPXBhaXI7CisgICAgICBjb25zdCBhY3Rpb249cm9sZT09PSJidXR0b24iPyJjbGljayI6CisgICAgICAgIHJvbGU9PT0idGV4dGJveCImJlsiVXNlcm5hbWUiLCJQYXNzd29yZCJdLmluY2x1ZGVzKG5hbWUpPyJ0eXBlIjpudWxsOworICAgICAgaWYoYWN0aW9uPT09bnVsbHx8IUFycmF5LmlzQXJyYXkobi5hY3Rpb25zKXx8IW4uYWN0aW9ucy5pbmNsdWRlcyhhY3Rpb24pKXJldHVybiBbXTsKKyAgICAgIHJldHVybiBbe3JvbGUsbmFtZSxhY3Rpb24scmVmfV07CisgICAgfSk6W10KKyAgfTsKK30KQEAgLTQzNSArNDYyLDAgQEAKLSAgY29uc3Qgbm9kZXM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudD8uY29udGVudF9yZWZzOwpAQCAtNDM4LDcgKzQ2NSwyIEBACi0gIGNvbnN0IHJvbGVzPVsiaGVhZGluZyIsImJ1dHRvbiIsInN0YXRpY3RleHQiLCJ0ZXh0Ym94Il07Ci0gIGNvbnN0IGxhYmVscz1bIlVzZXJuYW1lIiwiUGFzc3dvcmQiLCJBdXRoZW50aWNhdG9yIG9yIHJlY292ZXJ5IGNvZGUiLCJTaWduIGluIiwKLSAgICAiTG9jYWwgZGVtbyIsIlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiLAotICAgICJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZhaWxhYmxlLiJdOwotICByZXR1cm4gQXJyYXkuaXNBcnJheShub2RlcykmJnJvbGVzLmluY2x1ZGVzKHNwZWMuZXhwZWN0ZWRfcm9sZSkmJgotICAgIGxhYmVscy5pbmNsdWRlcyhzcGVjLmV4cGVjdGVkX2xhYmVsKSYmCi0gICAgbm9kZXMuc29tZShuPT5uLnJvbGU9PT1zcGVjLmV4cGVjdGVkX3JvbGUmJm4ubmFtZT09PXNwZWMuZXhwZWN0ZWRfbGFiZWwpOworICByZXR1cm4gcHJvamVjdFB1YmxpY0RlY2lzaW9uKHJlc3VsdCkub2JzZXJ2ZWQuc29tZShuPT4KKyAgICBuLnJvbGU9PT1zcGVjLmV4cGVjdGVkX3JvbGUmJm4ubmFtZT09PXNwZWMuZXhwZWN0ZWRfbGFiZWwpOwpAQCAtNDc4LDAgKzUwMSBAQAorICAgIHB1YmxpY19kZWNpc2lvbjpvd24ucHVibGljX2RlY2lzaW9uPz9udWxsLAo=",
  "candidate_sha256": "505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f",
  "baseline_sha256": "7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8",
  "diff_sha256": "b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2",
  "logic_sha256": "6ae4fd802a5515aade80b0b46cffe38030d9b4a46db427b32ed47fb9e20e583c",
  "edits": [
    {
      "id": "projection-page",
      "old": "  store(\"d01_immediate_owned_handles\",own);\n  if(flags.error_match)",
      "next": "  own.public_decision=projectPublicDecision(result);\n  store(\"d01_immediate_owned_handles\",own);\n  if(flags.error_match)"
    },
    {
      "id": "projection-predicate",
      "old": "function publicPredicate(result,spec) {\n  const nodes=result?.structuredContent?.content_refs;\n  // Root must pin one printed public expected label/role before that action.\n  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.\n  const roles=[\"heading\",\"button\",\"statictext\",\"textbox\"];\n  const labels=[\"Username\",\"Password\",\"Authenticator or recovery code\",\"Sign in\",\n    \"Local demo\",\"Protected application access\",\n    \"Signed in. Protected application access is available.\"];\n  return Array.isArray(nodes)&&roles.includes(spec.expected_role)&&\n    labels.includes(spec.expected_label)&&\n    nodes.some(n=>n.role===spec.expected_role&&n.name===spec.expected_label);\n}",
      "next": "function projectPublicDecision(result) {\n  const s=result?.structuredContent,nodes=Array.isArray(s?.content_refs)?s.content_refs:[];\n  const labels=[\n    [\"heading\",\"Local demo\"],[\"heading\",\"Sign in to continue to Local demo\"],\n    [\"heading\",\"Local demo wants to use your riAuth account\"],\n    [\"heading\",\"Continue to Local demo?\"],[\"heading\",\"Protected application access\"],\n    [\"button\",\"Sign in\"],[\"button\",\"Allow\"],[\"button\",\"Continue\"],\n    [\"textbox\",\"Username\"],[\"textbox\",\"Password\"],\n    [\"textbox\",\"Authenticator or recovery code (if enabled)\"],\n    [\"statictext\",\"Signed in. Protected application access is available.\"]\n  ];\n  const valid=result?.isError!==true&&s?.status===\"ok\"&&s?.snapshot?.complete===true&&\n    !nodes.some(n=>n.name===\"Local demo could not complete this request.\");\n  return {\n    observed:valid?labels.filter(([role,name])=>\n      nodes.some(n=>n.role===role&&n.name===name)).map(([role,name])=>({role,name})):[],\n    refs:valid&&Array.isArray(s?.refs)?s.refs.flatMap(n=>{\n      const pair=labels.find(([role,name])=>n?.role===role&&n.name===name),ref=n?.ref;\n      if(!pair||typeof ref!==\"string\"||!/^p[0-9]+:[0-9]+$/.test(ref))return [];\n      const [role,name]=pair;\n      const action=role===\"button\"?\"click\":\n        role===\"textbox\"&&[\"Username\",\"Password\"].includes(name)?\"type\":null;\n      if(action===null||!Array.isArray(n.actions)||!n.actions.includes(action))return [];\n      return [{role,name,action,ref}];\n    }):[]\n  };\n}\nfunction publicPredicate(result,spec) {\n  // Root must pin one printed public expected label/role before that action.\n  // No raw field value, URL, subject, query or arbitrary regex predicate is allowed.\n  return projectPublicDecision(result).observed.some(n=>\n    n.role===spec.expected_role&&n.name===spec.expected_label);\n}"
    },
    {
      "id": "projection-output",
      "old": "  text({result:\"browser_decision_observed_only\",page_flags:record.last_page_flags??null,\n    journey_credit:false",
      "next": "  text({result:\"browser_decision_observed_only\",page_flags:record.last_page_flags??null,\n    public_decision:own.public_decision??null,\n    journey_credit:false"
    }
  ],
  "collectors": {
    "CLOCK": {
      "b64": "aW1wb3J0IGpzb24sdGltZQpwcmludChqc29uLmR1bXBzKHsnbW9ub3RvbmljX25zJzpzdHIodGltZS5tb25vdG9uaWNfbnMoKSksJ3dhbGxfZXBvY2hfbnMnOnN0cih0aW1lLnRpbWVfbnMoKSl9KSkK",
      "bytes": 114,
      "sha256": "cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d"
    },
    "STOP": {
      "b64": "aW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN5cyx0aW1lCmM9anNvbi5sb2FkcyhzeXMuYXJndlsxXSkKY2xvY2s9eydtb25vdG9uaWNfbnMnOnN0cih0aW1lLm1vbm90b25pY19ucygpKSwnd2FsbF9lcG9jaF9ucyc6c3RyKHRpbWUudGltZV9ucygpKX0KcmVjb3JkPXsnc2NoZW1hJzoncmlhdXRoLmQwMS1maXJzdC1vYnNlcnZhdGlvbi92MScsJ2ZpcnN0X2ZhaWx1cmUnOmNbJ2ZpcnN0X2ZhaWx1cmUnXSwnZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyc6Y1snZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyddLCdmaXJzdF9ldmVudF9wcm92ZW4nOkZhbHNlLCdjbG9jayc6Y2xvY2t9CmV2ZW50X3N0YXRlPSd3cml0ZV91bmNvbmZpcm1lZCcKdHJ5OgogICAgZmQ9b3Mub3BlbihwYXRobGliLlBhdGgoY1snZXZlbnRfb3V0J10pLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApCiAgICB3aXRoIG9zLmZkb3BlbihmZCwndycsZW5jb2Rpbmc9J2FzY2lpJykgYXMgZjoKICAgICAgICBmLndyaXRlKGpzb24uZHVtcHMocmVjb3JkLHNvcnRfa2V5cz1UcnVlKSsnXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSkKICAgIGV2ZW50X3N0YXRlPSd3cml0dGVuJwpleGNlcHQgT1NFcnJvcjpwYXNzCiMgQXR0ZW1wdCBjbG9jayBwZXJzaXN0ZW5jZSBCRUZPUkUgbWFya2VyL3N0YXRlL2J1ZGdldCBjb21wYXJpc29ucy4KIyBBIG1ldGFkYXRhIGVycm9yIGRvZXMgbm90IHByZXZlbnQgdGhlIG9yaWdpbmFsIGVzc2VudGlhbCBzdG9wIHByb3RvY29sLgpsYWI9cGF0aGxpYi5QYXRoKGNbJ2xhYiddKTttYXJrZXJfc3RhdGU9J2xhYl9hYnNlbnQnCnRyeToKICAgIGlmIGxhYi5leGlzdHMoKToKICAgICAgICBpbmZvPWxhYi5sc3RhdCgpCiAgICAgICAgaWYgbm90IHN0YXQuU19JU0RJUihpbmZvLnN0X21vZGUpIG9yIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpIT0wbzcwMCBvciBpbmZvLnN0X3VpZCE9b3MuZ2V0dWlkKCk6CiAgICAgICAgICAgIG1hcmtlcl9zdGF0ZT0nb3duZXJzaGlwX3Vua25vd24nCiAgICAgICAgZWxzZToKICAgICAgICAgICAgbWFya2VyX3N0YXRlPSdzdG9wX3JlcXVlc3RlZCcKICAgICAgICAgICAgZm9yIG5hbWUgaW4gKCgndWktZmFpbHVyZScsJ3N0b3AnKSBpZiBjWydmaXJzdF9mYWlsdXJlJ10gaXMgbm90IE5vbmUgZWxzZSAoJ3N0b3AnLCkpOgogICAgICAgICAgICAgICAgdHJ5OgogICAgICAgICAgICAgICAgICAgIGZkPW9zLm9wZW4obGFiL25hbWUsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMCkKICAgICAgICAgICAgICAgICAgICBvcy5jbG9zZShmZCkKICAgICAgICAgICAgICAgIGV4Y2VwdCBGaWxlTm90Rm91bmRFcnJvcjptYXJrZXJfc3RhdGU9J2xhYl9hYnNlbnQnCiAgICAgICAgICAgICAgICBleGNlcHQgRmlsZUV4aXN0c0Vycm9yOnBhc3MKZXhjZXB0IE9TRXJyb3I6bWFya2VyX3N0YXRlPSdzdG9wX3VuY29uZmlybWVkJwpwcmludChqc29uLmR1bXBzKHsnY2xvY2snOmNsb2NrLCdldmVudF9zdGF0ZSc6ZXZlbnRfc3RhdGUsJ21hcmtlcl9zdGF0ZSc6bWFya2VyX3N0YXRlfSkpCg==",
      "bytes": 1600,
      "sha256": "5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071"
    },
    "READBACK": {
      "b64": "aW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN1YnByb2Nlc3Msc3lzLHRpbWUKYz1qc29uLmxvYWRzKHN5cy5hcmd2WzFdKQpjaGlsZHJlbj1Ob25lO2hlbHBlcl9waWQ9Y1snaGVscGVyX3BpZCddO291dGVyX29ic2VydmF0aW9uPU5vbmU7cHJvamVjdGlvbj0ndW5hdmFpbGFibGUnCmRlZiBmaW5pdGVfb2JzZXJ2YXRpb24odmFsdWUpOgogICAgaWYgdHlwZSh2YWx1ZSkgaXMgbm90IGRpY3Qgb3Igc2V0KHZhbHVlKSE9IHsnb2JzZXJ2ZWQnLCdkaWFnbm9zdGljJ30gb3IgdHlwZSh2YWx1ZVsnb2JzZXJ2ZWQnXSkgaXMgbm90IGJvb2w6CiAgICAgICAgcmV0dXJuIE5vbmUKICAgIGRpYWdub3N0aWM9dmFsdWVbJ2RpYWdub3N0aWMnXQogICAgaWYgZGlhZ25vc3RpYyBpcyBOb25lOgogICAgICAgIHJldHVybiB7J29ic2VydmVkJzp2YWx1ZVsnb2JzZXJ2ZWQnXSwnZGlhZ25vc3RpYyc6Tm9uZX0KICAgIGlmIG5vdCB2YWx1ZVsnb2JzZXJ2ZWQnXSBvciB0eXBlKGRpYWdub3N0aWMpIGlzIG5vdCBkaWN0IG9yIHNldChkaWFnbm9zdGljKSE9IHsnc2l0ZScsJ2V4Y2VwdGlvbl9jbGFzcycsJ293bl9mdW5jdGlvbicsJ293bl9saW5lJ306CiAgICAgICAgcmV0dXJuIE5vbmUKICAgIHNpdGVzPSgnaGFuZGxlcicsJ3NlcnZlcicsJ21haW4nKQogICAga2luZHM9KCdBdHRyaWJ1dGVFcnJvcicsJ1R5cGVFcnJvcicsJ1ZhbHVlRXJyb3InLCdLZXlFcnJvcicsJ09TRXJyb3InLCdCcm9rZW5QaXBlRXJyb3InLCdDb25uZWN0aW9uUmVzZXRFcnJvcicsJ1RpbWVvdXRFcnJvcicsJ290aGVyJykKICAgIGZ1bmN0aW9ucz0oJ0hlYWRlclJlYWRlci5yZWFkbGluZScsJ0RlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0JywnRGVtby5iZWdpbicsJ0RlbW8uY2FsbGJhY2snLCdEZW1vLmludm9rZScsJ0hhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0JywnSGFuZGxlci5zZW5kX2Vycm9yJywnSGFuZGxlci5nZXQnLCdIYW5kbGVyLnJlcGx5JywnbWFpbicpCiAgICBzaXRlLGtpbmQsZnVuY3Rpb24sbGluZT0oZGlhZ25vc3RpY1trXSBmb3IgayBpbiAoJ3NpdGUnLCdleGNlcHRpb25fY2xhc3MnLCdvd25fZnVuY3Rpb24nLCdvd25fbGluZScpKQogICAgaWYgdHlwZShzaXRlKSBpcyBub3Qgc3RyIG9yIHNpdGUgbm90IGluIHNpdGVzIG9yIHR5cGUoa2luZCkgaXMgbm90IHN0ciBvciBraW5kIG5vdCBpbiBraW5kczoKICAgICAgICByZXR1cm4gTm9uZQogICAgaWYgbm90ICgoZnVuY3Rpb24gaXMgTm9uZSBhbmQgbGluZSBpcyBOb25lKSBvciAodHlwZShmdW5jdGlvbikgaXMgc3RyIGFuZCBmdW5jdGlvbiBpbiBmdW5jdGlvbnMgYW5kIHR5cGUobGluZSkgaXMgaW50IGFuZCAxPD1saW5lPD0xMDI0KSk6CiAgICAgICAgcmV0dXJuIE5vbmUKICAgIHJldHVybiB7J29ic2VydmVkJzpUcnVlLCdkaWFnbm9zdGljJzp7J3NpdGUnOnNpdGUsJ2V4Y2VwdGlvbl9jbGFzcyc6a2luZCwnb3duX2Z1bmN0aW9uJzpmdW5jdGlvbiwnb3duX2xpbmUnOmxpbmV9fQpvdXRlcj1wYXRobGliLlBhdGgoY1snb3V0ZXJfb3V0J10pCmlmIG91dGVyLmlzX2ZpbGUoKToKICAgIGZkPW9zLm9wZW4ob3V0ZXIsb3MuT19SRE9OTFl8b3MuT19OT0ZPTExPV3xvcy5PX05PTkJMT0NLKQogICAgdHJ5OgogICAgICAgIGluZm89b3MuZnN0YXQoZmQpCiAgICAgICAgaWYgbm90IChzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKT09MG82MDAgYW5kIDA8aW5mby5zdF9zaXplPD0yNjIxNDQpOgogICAgICAgICAgICByYWlzZSBWYWx1ZUVycm9yKCdmaXhlZF9vdXRlcl9ldmlkZW5jZV9pbnZhbGlkJykKICAgICAgICB3aXRoIG9zLmZkb3BlbihmZCwncmInLGNsb3NlZmQ9RmFsc2UpIGFzIHNvdXJjZTpyYXdfYnl0ZXM9c291cmNlLnJlYWQoMjYyMTQ1KQogICAgICAgIGlmIG5vdCAwPGxlbihyYXdfYnl0ZXMpPD0yNjIxNDQ6cmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpCiAgICAgICAgZGF0YT1qc29uLmxvYWRzKHJhd19ieXRlcykKICAgIGZpbmFsbHk6b3MuY2xvc2UoZmQpCiAgICBvdXRlcl9vYnNlcnZhdGlvbj1maW5pdGVfb2JzZXJ2YXRpb24oZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbicpKQogICAgcHJvamVjdGlvbj1kYXRhLmdldCgndW5leHBlY3RlZF9vYnNlcnZhdGlvbl9wcm9qZWN0aW9uJykKICAgIGlmIHByb2plY3Rpb24gbm90IGluICgndW5hdmFpbGFibGUnLCd2YWxpZCcsJ2ludmFsaWQnKTpwcm9qZWN0aW9uPSdpbnZhbGlkJwogICAgaWYgcHJvamVjdGlvbj09J3ZhbGlkJyBhbmQgb3V0ZXJfb2JzZXJ2YXRpb24gaXMgTm9uZTpwcm9qZWN0aW9uPSdpbnZhbGlkJwogICAgYWxsb3dlZD17J3dob2FtaScsJ2Rpc2NvdmVyeScsJ2NvbmZpZGVudGlhbF9jbGllbnRfY3JlYXRlJywnb3BlcmF0b3JfbG9naW4nLCdzZXJ2ZXInLCdtYWludGVuYW5jZV9pbml0J30KICAgIHN0YXJ0ZWQ9ZGF0YS5nZXQoJ2hlbHBlcl9pbnZvY2F0aW9ucycpPT0xIGFuZCB0eXBlKGRhdGEuZ2V0KCdoZWxwZXJfcGlkJykpIGlzIGludCBhbmQgZGF0YVsnaGVscGVyX3BpZCddPjAKICAgIGlmIHN0YXJ0ZWQ6YWxsb3dlZC5hZGQoJ2hlbHBlcicpCiAgICByYXc9ZGF0YS5nZXQoJ293bmVkX2NoaWxkX2V4aXRzJykKICAgIGlmIGlzaW5zdGFuY2UocmF3LGxpc3QpIGFuZCBsZW4ocmF3KT09bGVuKGFsbG93ZWQpIGFuZCBhbGwoaXNpbnN0YW5jZSh2LGRpY3QpIGFuZCB2LmdldCgnbmFtZScpIGluIGFsbG93ZWQgYW5kIHR5cGUodi5nZXQoJ3BpZCcpKSBpcyBpbnQgYW5kIHZbJ3BpZCddPjAgYW5kIHR5cGUodi5nZXQoJ2V4aXQnKSkgaXMgaW50IGZvciB2IGluIHJhdykgYW5kIHt2WyduYW1lJ10gZm9yIHYgaW4gcmF3fT09YWxsb3dlZCBhbmQgbGVuKHt2WydwaWQnXSBmb3IgdiBpbiByYXd9KT09bGVuKGFsbG93ZWQpIGFuZCBuZXh0KHZbJ3BpZCddIGZvciB2IGluIHJhdyBpZiB2WyduYW1lJ109PSdzZXJ2ZXInKT09Y1snc2VydmVyX3BpZCddIGFuZCAoKHN0YXJ0ZWQgYW5kIG5leHQodlsncGlkJ10gZm9yIHYgaW4gcmF3IGlmIHZbJ25hbWUnXT09J2hlbHBlcicpPT1kYXRhWydoZWxwZXJfcGlkJ10gYW5kIChoZWxwZXJfcGlkIGlzIE5vbmUgb3IgaGVscGVyX3BpZD09ZGF0YVsnaGVscGVyX3BpZCddKSkgb3IgKG5vdCBzdGFydGVkIGFuZCBoZWxwZXJfcGlkIGlzIE5vbmUgYW5kIGRhdGEuZ2V0KCdoZWxwZXJfaW52b2NhdGlvbnMnKSBpcyBOb25lIGFuZCBkYXRhLmdldCgnaGVscGVyX3BpZCcpIGlzIE5vbmUpKToKICAgICAgICBjaGlsZHJlbj1be2s6dltrXSBmb3IgayBpbiAoJ25hbWUnLCdwaWQnLCdleGl0Jyl9IGZvciB2IGluIHJhd10KICAgICAgICBoZWxwZXJfcGlkPWRhdGFbJ2hlbHBlcl9waWQnXSBpZiBzdGFydGVkIGVsc2UgTm9uZQpwaWRzPWxpc3QoY1snb3duZWRfcGlkcyddKQppZiBoZWxwZXJfcGlkIGlzIG5vdCBOb25lIGFuZCBoZWxwZXJfcGlkIG5vdCBpbiBwaWRzOnBpZHMuYXBwZW5kKGhlbHBlcl9waWQpCnA9c3VicHJvY2Vzcy5ydW4oWycvYmluL3BzJywnLXAnLCcsJy5qb2luKHN0cih2KSBmb3IgdiBpbiBwaWRzKSwnLW8nLCdwaWQ9J10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpCnBzX2tub3duPXAucmV0dXJuY29kZSBpbiAoMCwxKSBhbmQgYWxsKHYuaXNkaWdpdCgpIGZvciB2IGluIHAuc3Rkb3V0LnNwbGl0KCkpCnByZXNlbnQ9c2V0KGludCh2KSBmb3IgdiBpbiBwLnN0ZG91dC5zcGxpdCgpKSBpZiBwc19rbm93biBlbHNlIHNldCgpCnBvcnRzPXt9CmZvciBwb3J0IGluICg5MDAwLDMwMDApOgogICAgcD1zdWJwcm9jZXNzLnJ1bihbJy91c3Ivc2Jpbi9sc29mJywnLW5QJywnLXQnLCctaVRDUDonK3N0cihwb3J0KSwnLXNUQ1A6TElTVEVOJ10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpCiAgICBwb3J0c1tzdHIocG9ydCldPW5vdCBib29sKHAuc3Rkb3V0LnN0cmlwKCkpIGlmIHAucmV0dXJuY29kZSBpbiAoMCwxKSBlbHNlIE5vbmUKcHJpbnQoanNvbi5kdW1wcyh7J2Nsb2NrJzp7J21vbm90b25pY19ucyc6c3RyKHRpbWUubW9ub3RvbmljX25zKCkpLCd3YWxsX2Vwb2NoX25zJzpzdHIodGltZS50aW1lX25zKCkpfSwnb3duZWRfcGlkcyc6e3N0cih2KToodiBub3QgaW4gcHJlc2VudCBpZiBwc19rbm93biBlbHNlIE5vbmUpIGZvciB2IGluIHBpZHN9LCdwb3J0cyc6cG9ydHMsJ2xhYl9hYnNlbnQnOm5vdCBwYXRobGliLlBhdGgoY1snbGFiJ10pLmV4aXN0cygpLCdvd25lZF9jaGlsZF9leGl0cyc6Y2hpbGRyZW4sJ2hlbHBlcl9waWQnOmhlbHBlcl9waWQsJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbic6b3V0ZXJfb2JzZXJ2YXRpb24sJ3VuZXhwZWN0ZWRfb2JzZXJ2YXRpb25fcHJvamVjdGlvbic6cHJvamVjdGlvbn0pKQo=",
      "bytes": 4424,
      "sha256": "b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f"
    },
    "MARKER": {
      "b64": "aW1wb3J0IG9zLHBhdGhsaWIsc3RhdCxzeXMKbGFiPXBhdGhsaWIuUGF0aChzeXMuYXJndlsxXSk7Z3VhcmQ9aW50KHN5cy5hcmd2WzJdKTtzZXJ2ZXI9aW50KHN5cy5hcmd2WzNdKQppZiBndWFyZDw9MCBvciBzZXJ2ZXI8PTAgb3IgZ3VhcmQ9PXNlcnZlcjpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKQppbmZvPWxhYi5sc3RhdCgpCmlmIG5vdCAoc3RhdC5TX0lTRElSKGluZm8uc3RfbW9kZSkgYW5kIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpPT0wbzcwMCBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpKTpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKQpvcy5raWxsKGd1YXJkLDApO29zLmtpbGwoc2VydmVyLDApCmlmIChsYWIvJ3N0b3AnKS5leGlzdHMoKSBvciAobGFiLyd1aS1mYWlsdXJlJykuZXhpc3RzKCk6cmFpc2UgVmFsdWVFcnJvcignb3duZWRfY29udGV4dF9pbnZhbGlkJykKZmQ9b3Mub3BlbihsYWIvJ2Jyb3dzZXItcHJlcGFyZWQnLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApCm9zLmNsb3NlKGZkKQpwcmludCgneyJwcmVwYXJlZF9tYXJrZXJfd3JpdHRlbiI6dHJ1ZX0nKQo=",
      "bytes": 626,
      "sha256": "13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949"
    },
    "PERSIST": {
      "b64": "aW1wb3J0IGpzb24sb3MscGF0aGxpYixzeXMKcGF0aD1wYXRobGliLlBhdGgoc3lzLmFyZ3ZbMV0pCnJlY29yZD1qc29uLmxvYWRzKHN5cy5hcmd2WzJdKQojIENvbnZlcnQgZGVjaW1hbCBzdHJpbmdzIGRpcmVjdGx5IHRvIFB5dGhvbiBpbnRlZ2VycywgYXZvaWRpbmcgSlMgTnVtYmVyIHJvdW5kaW5nLgpkZWYgY2xvY2tzKHZhbHVlKToKICAgIGlmIGlzaW5zdGFuY2UodmFsdWUsZGljdCk6CiAgICAgICAgZm9yIGssdiBpbiBsaXN0KHZhbHVlLml0ZW1zKCkpOgogICAgICAgICAgICBpZiBrIGluICgnbW9ub3RvbmljX25zJywnd2FsbF9lcG9jaF9ucycpIGFuZCBpc2luc3RhbmNlKHYsc3RyKToKICAgICAgICAgICAgICAgIGFzc2VydCB2LmlzZGVjaW1hbCgpIGFuZCBsZW4odik8PTE5IGFuZCAwPD1pbnQodik8MioqNjMKICAgICAgICAgICAgICAgIHZhbHVlW2tdPWludCh2KQogICAgICAgICAgICBlbHNlOmNsb2Nrcyh2KQogICAgZWxpZiBpc2luc3RhbmNlKHZhbHVlLGxpc3QpOgogICAgICAgIGZvciB2IGluIHZhbHVlOmNsb2Nrcyh2KQpjbG9ja3MocmVjb3JkKQpmZD1vcy5vcGVuKHBhdGgsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMCkKd2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6CiAgICBmLndyaXRlKGpzb24uZHVtcHMocmVjb3JkLHNvcnRfa2V5cz1UcnVlLGluZGVudD0yKSsnXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSkKcHJpbnQoanNvbi5kdW1wcyh7J3dyaXR0ZW5fZXhjbHVzaXZlJzpUcnVlfSkpCg==",
      "bytes": 805,
      "sha256": "819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30"
    }
  },
  "case_plan": [
    {
      "name": "phase_entry_then_continuation",
      "group": "phase_binding"
    },
    {
      "name": "password_survives_until_password_dispatch",
      "group": "secret_lifetime"
    },
    {
      "name": "missing_password_refuses_without_input",
      "group": "secret_lifetime"
    },
    {
      "name": "unowned_context_refuses_without_operations",
      "group": "phase_binding"
    },
    {
      "name": "undeclared_ref_refuses_input",
      "group": "phase_binding"
    },
    {
      "name": "stale_ref_cannot_cross_cells",
      "group": "phase_binding"
    },
    {
      "name": "invalid_kind_repeated_cleanup_clears_before_await",
      "group": "secret_lifetime"
    },
    {
      "name": "helper_zero_final_snapshot_then_cleanup",
      "group": "helper_handoff"
    },
    {
      "name": "helper_zero_missing_page_still_fails",
      "group": "helper_handoff"
    },
    {
      "name": "helper_nonzero_final_still_refuses",
      "group": "helper_handoff"
    },
    {
      "name": "helper_zero_other_kind_still_refuses",
      "group": "helper_handoff"
    },
    {
      "name": "helper_zero_prepared_phase_still_refuses",
      "group": "helper_handoff"
    },
    {
      "name": "partial_line_drains_before_output_and_next_cell",
      "group": "partial_framing"
    },
    {
      "name": "partial_exact_cap_is_accepted",
      "group": "partial_framing"
    },
    {
      "name": "partial_over_cap_refuses",
      "group": "partial_framing"
    },
    {
      "name": "partial_joined_controller_refuses",
      "group": "partial_framing"
    },
    {
      "name": "partial_unavailable_controller_refuses",
      "group": "partial_framing"
    },
    {
      "name": "partial_deadline_prevents_extra_poll",
      "group": "partial_framing"
    },
    {
      "name": "partial_completed_late_still_refuses",
      "group": "partial_framing"
    },
    {
      "name": "retained_start_budget_refuses_next_cell",
      "group": "phase_binding"
    },
    {
      "name": "inclusive_cleanup_deadline_does_not_invent_join",
      "group": "cleanup_latch"
    },
    {
      "name": "first_page_failure_survives_later_helper_error",
      "group": "cleanup_latch"
    },
    {
      "name": "missing_absence_proof_prevents_release",
      "group": "cleanup_latch"
    },
    {
      "name": "cleanup_exception_is_private",
      "group": "cleanup_latch"
    },
    {
      "name": "pending_metadata_command_prevents_release",
      "group": "cleanup_latch"
    },
    {
      "name": "projection_exact_pairs_and_private_field_omission",
      "group": "public_projection"
    },
    {
      "name": "projection_otp_observed_without_action_or_value",
      "group": "public_projection"
    },
    {
      "name": "projection_required_consent_allow_roundtrip",
      "group": "public_projection"
    },
    {
      "name": "projection_optional_consent_continue_roundtrip",
      "group": "public_projection"
    },
    {
      "name": "projection_role_and_label_rejections",
      "group": "public_projection"
    },
    {
      "name": "projection_incomplete_and_error_rejections",
      "group": "public_projection"
    },
    {
      "name": "projection_missing_or_wrong_advertised_action",
      "group": "public_projection"
    },
    {
      "name": "projection_malformed_refs",
      "group": "public_projection"
    },
    {
      "name": "projection_disjoint_rows_do_not_infer_association",
      "group": "public_projection"
    },
    {
      "name": "projection_duplicate_actions_preserve_multiplicity",
      "group": "public_projection"
    },
    {
      "name": "projection_latest_success_replaces_previous_snapshot",
      "group": "public_projection"
    }
  ],
  "check_names": [
    "real_deadline",
    "source_encoding",
    "privacy_sink",
    "diff_framing",
    "diff_headers",
    "diff_hunk",
    "diff_position",
    "diff_line",
    "diff_exact_line",
    "diff_hunk_count",
    "candidate_identity",
    "baseline_identity",
    "diff_identity",
    "edit_unique",
    "baseline_inverse",
    "collector_identity",
    "collector_in_candidate",
    "collector_set",
    "trace_cap",
    "call_cap",
    "store_key",
    "password_store_value",
    "receipt_precedes_comparison",
    "ready_receipt_order",
    "load_key",
    "collector_command",
    "collector_quoting",
    "collector_allowlist",
    "collector_arguments",
    "marker_bound",
    "clear_before_cleanup_await",
    "latch_before_cleanup",
    "clear_survives_cleanup_await",
    "readback_bound",
    "persist_bound",
    "bound_browser_handles",
    "bound_reference",
    "bound_controller",
    "navigation_allowlist",
    "snapshot_public_only",
    "click_route",
    "type_replace",
    "type_value",
    "kill_bound_owned_pid",
    "end_bound_session",
    "windows_bound_pid",
    "cell_cap",
    "cell_output",
    "no_whole60_credit",
    "no_journey_credit",
    "observed_only",
    "first_failure_matches",
    "release_separate",
    "single_cleanup_protocol",
    "password_cleared",
    "fixture_line_size",
    "entry_phase_and_helper",
    "ready_receipt_proved",
    "startup_budget_retained",
    "secret_survives_nonpassword",
    "input_route_sequence",
    "password_dispatch_consumes_once",
    "no_input_on_refusal",
    "unowned_refusal",
    "fresh_ref_replaces_consumed",
    "single_use_ref",
    "repeat_clear_no_repeat_await",
    "read_only_handoff",
    "zero_only_declared_final",
    "one_final_snapshot",
    "final_page_proved",
    "prepared_phase_refusal",
    "partial_event_processed_before_output",
    "no_partial_store",
    "no_partial_carry_next_cell",
    "no_extra_active_poll_at_deadline",
    "no_snapshot_after_deadline",
    "no_join_credit_at_inclusive_limit",
    "driver_failure_retained",
    "unique_case_names",
    "unexpected",
    "metadata_cap",
    "projection_action_pair",
    "projection_canonical_pair",
    "projection_canonical_ref",
    "projection_closed_shape",
    "projection_complete_cell_count",
    "projection_consent_dispatch_and_freshness",
    "projection_exact_output_and_store",
    "projection_failure_no_public_decision_or_input",
    "projection_latest_ref_only",
    "projection_no_automatic_input",
    "projection_no_cross_snapshot_join",
    "projection_old_ref_refused",
    "projection_processed_failure_empty",
    "projection_single_use_before_await"
  ],
  "diff_headers": [
    "--- a/archived-d01-continuation-cell.js\n",
    "+++ b/archived-d01-continuation-cell.js\n"
  ],
  "diff_hunks": 5
};

const LIMIT_MS=30000, MAX_CELLS=75, MAX_CALLS=2500, MAX_TRACE=512;
const PASSWORD="D01_SYNTHETIC_PASSWORD_SENTINEL_0123456789ABCDEFG";
const PARTIAL="D01_SYNTHETIC_PARTIAL_SENTINEL_PRIVATE";
const EXCEPTION="D01_SYNTHETIC_EXCEPTION_SENTINEL_PRIVATE";
const PROJECTION="D01_SYNTHETIC_PROJECTION_SENTINEL_PRIVATE";
const PRIVATE=[PASSWORD,PARTIAL,EXCEPTION,PROJECTION];
const OBS="d01_immediate_observation_record", OWN="d01_immediate_owned_handles";
const PW="d01_fresh_password_input", EXIT=Object.freeze({});
const FAULTS=new WeakMap();
const result={schema:"riauth.d01-continuation-memory/v1",
  source:null,planned:BINDING.case_plan,attempted:[],completed:[],
  completed_groups:{},cells:0,stub_counts:{},assertions_completed:0,
  privacy_checks_completed:0,first_failure:null,unreached:[],elapsed_ms:null,
  full_candidate_only:true,actual_tools_or_product:false};
function failure(code){const e=Object.freeze({});FAULTS.set(e,code);return e;}
function ensure(ok,code){if(!ok)throw failure(code);result.assertions_completed++;}
function clockGuard(){ensure(performance.now()<LIMIT_MS,"real_deadline");}
const clone=v=>v===undefined?undefined:JSON.parse(JSON.stringify(v));
const sha=s=>createHash("sha256").update(s).digest("hex");
function decode(encoded){ensure(typeof encoded==="string"&&
  /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(encoded),
  "source_encoding");const b=Buffer.from(encoded,"base64");
  ensure(b.toString("base64")===encoded,"source_encoding");return b.toString("utf8");}
function privacy(value){
  const s=JSON.stringify(value);ensure(typeof s==="string"&&!PRIVATE.some(x=>s.includes(x)),
    "privacy_sink");result.privacy_checks_completed++;
}
function inverseDiff(candidate,diff){
  const lines=candidate.match(/[^\n]*\n/g)||[], ds=diff.match(/[^\n]*\n/g)||[];
  ensure(lines.join("")===candidate&&ds.join("")===diff,"diff_framing");
  let cursor=0,i=2,out=[],hunks=0;
  ensure(ds[0]===BINDING.diff_headers[0]&&
    ds[1]===BINDING.diff_headers[1],"diff_headers");
  while(i<ds.length){
    const m=/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n$/.exec(ds[i++]);
    ensure(m!==null,"diff_hunk");hunks++;const count=m[4]===undefined?1:Number(m[4]);
    const start=count===0?Number(m[3]):Number(m[3])-1;
    ensure(cursor<=start,"diff_position");out.push(...lines.slice(cursor,start));cursor=start;
    while(i<ds.length&&!ds[i].startsWith("@@ ")){
      const kind=ds[i][0],text=ds[i++].slice(1);
      ensure([" ","+","-"].includes(kind),"diff_line");
      if(kind===" "||kind==="+"){ensure(lines[cursor]===text,"diff_exact_line");cursor++;}
      if(kind===" "||kind==="-")out.push(text);
    }
  }
  ensure(hunks===BINDING.diff_hunks,"diff_hunk_count");out.push(...lines.slice(cursor));return out.join("");
}
let CANDIDATE,BASELINE,SCRIPT,COLLECTORS;
function bindSource(){
  CANDIDATE=decode(BINDING.candidate_b64);BASELINE=decode(BINDING.baseline_b64);
  const diff=decode(BINDING.diff_b64);
  ensure(Buffer.byteLength(CANDIDATE)===33701&&sha(CANDIDATE)===BINDING.candidate_sha256,
    "candidate_identity");
  ensure(Buffer.byteLength(BASELINE)===32502&&sha(BASELINE)===BINDING.baseline_sha256,
    "baseline_identity");
  ensure(Buffer.byteLength(diff)===2405&&sha(diff)===BINDING.diff_sha256,"diff_identity");
  let restored=CANDIDATE;
  for(const e of [...BINDING.edits].reverse()){
    ensure(restored.split(e.next).length===2,"edit_unique");
    restored=restored.replace(e.next,e.old);
  }
  ensure(restored===BASELINE&&inverseDiff(CANDIDATE,diff)===BASELINE,"baseline_inverse");
  COLLECTORS=Object.fromEntries(Object.entries(BINDING.collectors).map(([name,v])=>{
    const source=decode(v.b64);
    ensure(Buffer.byteLength(source)===v.bytes&&sha(source)===v.sha256,"collector_identity");
    const prefix="const "+name+"=";
    const line=CANDIDATE.split("\n").find(x=>x.startsWith(prefix));
    ensure(typeof line==="string"&&line.endsWith(";")&&
      JSON.parse(line.slice(prefix.length,-1))===source,"collector_in_candidate");
    return [name,source];
  }));
  ensure(Object.keys(COLLECTORS).sort().join(",")==="CLOCK,MARKER,PERSIST,READBACK,STOP",
    "collector_set");
  SCRIPT=new vm.Script("(async function(){\n"+CANDIDATE+"\n})()",
    {filename:"reviewed-d01-memory-cell.js",displayErrors:false});
  result.source={candidate_sha256:sha(CANDIDATE),candidate_bytes:Buffer.byteLength(CANDIDATE),
    baseline_sha256:sha(BASELINE),baseline_bytes:Buffer.byteLength(BASELINE),
    diff_sha256:sha(diff),logic_sha256:BINDING.logic_sha256,full_inverse:true};
}
function event(value){return JSON.stringify(value)+"\n";}
function publicPage(kind,ref){
  const nodes=kind==="after"?
    [{role:"heading",name:"Protected application access"},
     {role:"statictext",name:"Signed in. Protected application access is available."}]:
    kind==="before"?[{role:"heading",name:"Local demo"},{role:"statictext",name:"Sign in required."}]:
    kind==="application"?[{role:"heading",name:"Local demo"},
      {role:"statictext",name:"Use Sign in to open this local application."},
      {role:"button",name:"Sign in"}]:
    [{role:"textbox",name:"Username"},{role:"textbox",name:"Password"},
     {role:"button",name:"Sign in"}];
  const base=Number(ref.split(":")[1]),prefix=ref.split(":")[0]+":";
  const refs=kind==="after"||kind==="before"?[]:
    kind==="application"?[{role:"button",name:"Sign in",ref,actions:["click"]}]:[
      {role:"textbox",name:"Username",ref,actions:["type"]},
      {role:"textbox",name:"Password",ref:prefix+String(base+1000),actions:["type"]},
      {role:"button",name:"Sign in",ref:prefix+String(base+2000),actions:["click"]}
    ];
  return {structuredContent:{status:"ok",snapshot:{complete:true},content_refs:nodes,refs}};
}
function makeState(options={}){
  const own={runtime_release:true,driver_owned:true,exact_bound:true,single_controller:true,
    phase:"browser_decision",prepared_gate:true,fixture_ready:true,listener_pid_proof:true,
    fresh_paths_preflight:true,guard_pid:11,server_pid:12,browser_pid:13,helper_pid:14,
    exec_session:15,start_epoch_ms:1000000,workspace:"/synthetic/workspace",
    session:"synthetic-session",target_id:"synthetic-target",tab_id:"synthetic-tab",
    lab:"/synthetic/lab",outer_out:"/synthetic/outer",event_out:"/synthetic/event",
    cleanup_out:"/synthetic/cleanup",fresh_refs:["p1:1"],next_decision:null};
  if(options.entry){own.phase="prepared_entry";own.helper_pid=null;
    own.fixture_ready=false;own.listener_pid_proof=false;}
  const s={map:new Map([[OWN,clone(own)]]),now:own.start_epoch_ms,queue:[],
    outputs:[],metadata:[],trace:[],snapshotQueue:[],inputKinds:[],clearEvents:[],
    stopCount:0,stopProofs:[],pollCount:0,lastPollReturn:0,lastReceipt:0,
    receiptReadyProofs:0,refCounter:1,cleanup:false,options,firstHostFault:null};
  if(options.password)s.map.set(PW,PASSWORD);
  return s;
}
function trace(s,kind,data={}){
  clockGuard();ensure(s.trace.length<MAX_TRACE,"trace_cap");
  result.stub_counts[kind]=(result.stub_counts[kind]||0)+1;
  ensure(Object.values(result.stub_counts).reduce((a,b)=>a+b,0)<=MAX_CALLS,"call_cap");
  const entry={kind,...data};privacy(entry);s.trace.push(entry);return s.trace.length;
}
function store(s,key,value){
  ensure([OWN,OBS,PW].includes(key),"store_key");
  if(key!==PW)privacy(value);
  if(key===PW){ensure(value===null||value===PASSWORD,"password_store_value");
    if(value===null)s.clearEvents.push(s.trace.length);}
  if(key===OBS){
    const old=s.map.get(OBS),n=value.controller_observations.length;
    if(n>(old?.controller_observations.length??0)){
      ensure(s.trace.length<MAX_TRACE,"trace_cap");
      s.lastReceipt=s.trace.length+1;
      s.trace.push({kind:"retained_controller_receipt"});
      ensure(s.lastReceipt>s.lastPollReturn,"receipt_precedes_comparison");
    }
  }
  if(key===OWN&&value.fixture_ready===true&&s.map.get(OWN)?.fixture_ready!==true){
    ensure(s.lastReceipt>s.lastPollReturn&&s.lastReceipt>0,"ready_receipt_order");
    s.receiptReadyProofs++;
  }
  s.map.set(key,clone(value));
}
function load(s,key){ensure([OWN,OBS,PW].includes(key),"load_key");return clone(s.map.get(key));}
function decision(s,kind,ref){
  const o=load(s,OWN);o.next_decision={kind,expected_role:"textbox",expected_label:"Username"};
  if(ref!==undefined)o.next_decision.ref=ref;
  s.map.set(OWN,o);
}
function quotedArgs(cmd){
  ensure(typeof cmd==="string"&&cmd.startsWith("python3 -c "),"collector_command");
  let i=11,out=[];
  while(i<cmd.length){
    ensure(cmd[i]==="'","collector_quoting");i++;let text="";
    for(;;){
      ensure(i<cmd.length,"collector_quoting");
      if(cmd.startsWith("'\\''",i)){text+="'";i+=4;continue;}
      if(cmd[i]==="'"){i++;break;}text+=cmd[i++];
    }
    out.push(text);
    if(i===cmd.length)break;
    ensure(cmd[i]===" ","collector_quoting");i++;
  }
  return out;
}
async function local(s,args){
  const argv=quotedArgs(args.cmd),source=argv.shift();
  const label=Object.keys(COLLECTORS).find(k=>COLLECTORS[k]===source);
  ensure(label!==undefined&&args.workdir==="/synthetic/workspace","collector_allowlist");
  trace(s,"collector_"+label.toLowerCase());
  const o=load(s,OWN),rec=load(s,OBS),clock={
    monotonic_ns:String(BigInt(s.now)*1000000n),wall_epoch_ns:String(BigInt(s.now)*1000000n)};
  let data;
  if(label==="CLOCK"){ensure(argv.length===0,"collector_arguments");data=clock;}
  else if(label==="MARKER"){
    ensure(argv.length===3&&argv[0]===o.lab&&argv[1]==="11"&&argv[2]==="12","marker_bound");
    data={prepared_marker_written:true};
  } else if(label==="STOP"){
    ensure(argv.length===1,"collector_arguments");const c=JSON.parse(argv[0]);privacy(c);
    ensure(load(s,PW)===null,"clear_before_cleanup_await");
    ensure(c.first_failure===rec.first_failure&&
      c.first_observation_wall_ms===rec.first_observation_wall_ms,"latch_before_cleanup");
    s.stopProofs.push(true);s.stopCount++;s.cleanup=true;
    await Promise.resolve();ensure(load(s,PW)===null,"clear_survives_cleanup_await");
    data={clock,event_state:"written",marker_state:"stop_requested"};
  } else if(label==="READBACK"){
    ensure(argv.length===1,"collector_arguments");const c=JSON.parse(argv[0]);privacy(c);
    ensure(c.lab===o.lab&&c.server_pid===12&&c.helper_pid===o.helper_pid,"readback_bound");
    const names=["whoami","discovery","confidential_client_create","operator_login","server","maintenance_init"];
    const pids=[21,22,23,24,12,25];if(o.helper_pid!==null){names.push("helper");pids.push(14);}
    data={clock,owned_pids:Object.fromEntries(c.owned_pids.map(p=>[String(p),true])),
      ports:{"9000":true,"3000":true},lab_absent:!s.options.absence_missing,
      owned_child_exits:names.map((name,i)=>({name,pid:pids[i],exit:0})),
      helper_pid:o.helper_pid,unexpected_failure_observation:{observed:false,diagnostic:null},
      unexpected_observation_projection:"valid"};
  } else {
    ensure(argv.length===2&&argv[0]===o.cleanup_out,"persist_bound");
    const record=JSON.parse(argv[1]);privacy(record);
    ensure(Buffer.byteLength(JSON.stringify(record))<=262144,"metadata_cap");s.metadata.push(clone(record));
    if(s.options.metadata_pending)return {session_id:99,output:""};
    data={written_exclusive:true};
  }
  return {exit_code:0,output:JSON.stringify(data)};
}
function bound(s,args,ref=false){
  const o=load(s,OWN);ensure(args.session===o.session&&args.target_id===o.target_id&&
    args.tab_id===o.tab_id,"bound_browser_handles");
  if(ref)ensure(typeof args.ref==="string"&&/^p[0-9]+:[0-9]+$/.test(args.ref),"bound_reference");
}
async function poll(s,args){
  ensure(args.session_id===15&&args.chars==="","bound_controller");
  trace(s,"controller_poll",{cleanup:s.cleanup});s.pollCount++;
  let r;
  if(s.cleanup){
    r={exit_code:0,output:event(s.options.cleanup_helper_error?
      {helper_completed:true,exit:1,fixture_finished:true}:{fixture_finished:true})};
  } else {
    const next=s.queue.shift()??{};
    if(next.throw){throw new Error(EXCEPTION);}
    if(next.at!==undefined)s.now=1000000+next.at;
    r={output:next.output??"",...(next.exit===undefined?{}:{exit_code:next.exit})};
  }
  s.lastPollReturn=s.trace.length;return r;
}
function bridge(s,operation){
  const remember=e=>{if(FAULTS.has(e)&&s.firstHostFault===null)s.firstHostFault=e;throw e;};
  try{const v=operation();return v instanceof Promise?v.catch(remember):v;}
  catch(e){return remember(e);}
}
function tools(s){
  const methods={
    exec_command:a=>local(s,a),write_stdin:a=>poll(s,a),
    mcp__cua_driver__browser_navigate:async a=>{
      bound(s,a);ensure(["http://localhost:3000/","http://localhost:3000/protected"].includes(a.url),
        "navigation_allowlist");trace(s,"navigate");return {structuredContent:{status:"ok"}};
    },
    mcp__cua_driver__get_browser_state:async a=>{
      bound(s,a);ensure(a.snapshot_format==="semantic_v2"&&a.include_screenshot===false,
        "snapshot_public_only");trace(s,"snapshot");
      const kind=s.snapshotQueue.shift()??"generic";const ref="p1:"+String(++s.refCounter);
      if(kind!==null&&typeof kind==="object")return clone(kind);
      if(kind==="missing")return {structuredContent:{status:"ok",snapshot:{complete:true},
        content_refs:[],refs:[]}};
      return publicPage(kind,ref);
    },
    mcp__cua_driver__browser_click:async a=>{
      bound(s,a,true);ensure(a.input_route==="dom_event","click_route");
      if(s.options.projection_protocol)ensure(load(s,OWN).fresh_refs.length===0&&
        a.ref===s.options.expected_action_ref,"projection_single_use_before_await");
      trace(s,"click");s.inputKinds.push("click");return {structuredContent:{effect:"confirmed"}};
    },
    mcp__cua_driver__browser_type:async a=>{
      bound(s,a,true);ensure(a.replace===true,"type_replace");
      const kind=a.text===PASSWORD?"password":a.text==="admin"?"username":null;
      ensure(kind!==null,"type_value");trace(s,"type",{kind,password_matches:kind==="password"});
      s.inputKinds.push(kind);return {structuredContent:{effect:"confirmed"}};
    },
    mcp__cua_driver__kill_app:async a=>{
      ensure(a.pid===13,"kill_bound_owned_pid");trace(s,"kill");
      if(s.options.driver_exception)throw new Error(EXCEPTION);
      return {structuredContent:{status:"ok"}};
    },
    mcp__cua_driver__end_session:async a=>{
      ensure(a.session==="synthetic-session","end_bound_session");trace(s,"end_session");
      return {structuredContent:{active:false,session:a.session}};
    },
    mcp__cua_driver__list_windows:async a=>{
      ensure(a.pid===13,"windows_bound_pid");trace(s,"windows");
      return {structuredContent:{windows:[]}};
    }
  };
  return Object.freeze(Object.fromEntries(Object.entries(methods)
    .map(([name,f])=>[name,a=>bridge(s,()=>f(a))])));
}
async function cell(s){
  clockGuard();ensure(result.cells<MAX_CELLS,"cell_cap");result.cells++;
  const sandbox=Object.create(null);
  Object.assign(sandbox,{tools:tools(s),store:(k,v)=>bridge(s,()=>store(s,k,v)),
    load:k=>bridge(s,()=>load(s,k)),text:v=>bridge(s,()=>{privacy(v);s.outputs.push(clone(v));}),
    exit:()=>{throw EXIT;},Date:Object.freeze({now:()=>s.now})});
  const ctx=vm.createContext(sandbox,{codeGeneration:{strings:false,wasm:false}});
  let timer;
  try{
    const remaining=Math.max(1,Math.floor(LIMIT_MS-performance.now()));
    const execution=SCRIPT.runInContext(ctx,{timeout:remaining,displayErrors:false});
    await Promise.race([Promise.resolve(execution),new Promise((_,reject)=>{
      timer=setTimeout(()=>reject(failure("real_deadline")),remaining);
    })]);
  }catch(e){if(e!==EXIT)throw e;}
  finally{if(timer!==undefined)clearTimeout(timer);}
  clockGuard();if(s.firstHostFault!==null)throw s.firstHostFault;
  ensure(s.outputs.length>0,"cell_output");
  for(const [k,v] of s.map)if(k!==PW)privacy(v);
  privacy(s.outputs);privacy(s.metadata);privacy(s.trace);
  return s.outputs.at(-1);
}
function finalOutcome(s,expected,release){
  const o=s.outputs.at(-1);privacy(o);
  ensure(o.whole_cleanup_within60_proven===false,"no_whole60_credit");
  ensure(o.journey_credit!==true,"no_journey_credit");
  if(expected===null)ensure(o.result==="browser_decision_observed_only","observed_only");
  else ensure(o.result==="failed"&&o.first_failure===expected,"first_failure_matches");
  if(release!==undefined)ensure(o.resource_release_proven===release,"release_separate");
  if(s.stopCount){ensure(s.stopCount===1&&s.stopProofs.length===1,"single_cleanup_protocol");
    ensure(load(s,PW)===null,"password_cleared");}
}
function firstActive(s,output="",extra={}){s.queue.push({output,...extra});}
function setTime(s,ms){s.now=1000000+ms;}
function partialEvent(length){
  const fixed={unexpected_failure_observation:{observed:true,diagnostic:null},
    unexpected_observation_projection:"valid",opaque:PARTIAL,pad:""};
  let line=event(fixed);if(length!==undefined){fixed.pad="x".repeat(length-line.length);line=event(fixed);}
  ensure(length===undefined||line.length===length,"fixture_line_size");
  return line;
}
const PUBLIC_PAIRS=[
  ["heading","Local demo"],["heading","Sign in to continue to Local demo"],
  ["heading","Local demo wants to use your riAuth account"],
  ["heading","Continue to Local demo?"],["heading","Protected application access"],
  ["button","Sign in"],["button","Allow"],["button","Continue"],
  ["textbox","Username"],["textbox","Password"],
  ["textbox","Authenticator or recovery code (if enabled)"],
  ["statictext","Signed in. Protected application access is available."]
];
function projectionNode(role,name){
  return {role,name,value:PROJECTION,url:PROJECTION,subject:PROJECTION,
    state:PROJECTION,cookie:PROJECTION,headers:{authorization:PROJECTION},error:PROJECTION};
}
function projectionRef(role,name,ref,actions){
  return {...projectionNode(role,name),ref,actions};
}
function projectionSnapshot(nodes,refs,changes={}){
  return {structuredContent:{status:"ok",snapshot:{complete:true},
    content_refs:nodes,refs,...changes}};
}
function projectionDecision(s,kind,role,name,ref){
  decision(s,kind,ref);const o=load(s,OWN);
  o.next_decision.expected_role=role;o.next_decision.expected_label=name;s.map.set(OWN,o);
}
function keysExactly(value,keys){
  return value!==null&&typeof value==="object"&&!Array.isArray(value)&&
    Object.keys(value).sort().join(",")===[...keys].sort().join(",");
}
function projectionShape(value){
  ensure(keysExactly(value,["observed","refs"])&&
    Array.isArray(value.observed)&&Array.isArray(value.refs),"projection_closed_shape");
  for(const row of value.observed){
    ensure(keysExactly(row,["role","name"])&&PUBLIC_PAIRS.some(([role,name])=>
      row.role===role&&row.name===name),"projection_canonical_pair");
  }
  for(const row of value.refs){
    ensure(keysExactly(row,["role","name","action","ref"])&&PUBLIC_PAIRS.some(([role,name])=>
      row.role===role&&row.name===name)&&typeof row.ref==="string"&&
      /^p[0-9]+:[0-9]+$/.test(row.ref),"projection_canonical_ref");
    ensure((row.role==="button"&&row.action==="click")||
      (row.role==="textbox"&&["Username","Password"].includes(row.name)&&row.action==="type"),
      "projection_action_pair");
  }
  privacy(value);
}
function projected(s,out,expected){
  finalOutcome(s,null,false);projectionShape(out.public_decision);
  const owned=load(s,OWN);projectionShape(owned.public_decision);
  ensure(JSON.stringify(out.public_decision)===JSON.stringify(expected)&&
    JSON.stringify(owned.public_decision)===JSON.stringify(expected),"projection_exact_output_and_store");
  return out;
}
function projectionFailed(s,out,expected,emptyOwned=true){
  finalOutcome(s,expected,true);
  ensure(!Object.hasOwn(out,"public_decision")&&s.inputKinds.length===0,
    "projection_failure_no_public_decision_or_input");
  if(emptyOwned)ensure(JSON.stringify(load(s,OWN).public_decision)===
    '{"observed":[],"refs":[]}',"projection_processed_failure_empty");
}
function noProjectionInput(s){
  ensure(s.inputKinds.length===0&&!s.trace.some(x=>["click","type"].includes(x.kind)),
    "projection_no_automatic_input");
}

async function runCase(name){
  let s;
  switch(name){
    case "phase_entry_then_continuation":{
      s=makeState({entry:true});s.snapshotQueue=["before","application"];
      firstActive(s,event({fixture_ready:true,guard_pid:11,server_pid:12,helper_pid:14,lab:"/synthetic/lab"}));
      await cell(s);ensure(load(s,OWN).phase==="browser_decision"&&load(s,OWN).helper_pid===14,
        "entry_phase_and_helper");ensure(s.receiptReadyProofs===1,"ready_receipt_proved");
      const start=load(s,OWN).start_epoch_ms;decision(s,"snapshot");await cell(s);
      ensure(load(s,OWN).start_epoch_ms===start,"startup_budget_retained");finalOutcome(s,null,false);break;
    }
    case "password_survives_until_password_dispatch":{
      s=makeState({password:true});
      for(const kind of ["username","click","snapshot"]){
        const ref=load(s,OWN).fresh_refs[0];decision(s,kind,kind==="snapshot"?undefined:ref);
        await cell(s);ensure(load(s,PW)===PASSWORD&&s.clearEvents.length===0,"secret_survives_nonpassword");
      }
      decision(s,"password",load(s,OWN).fresh_refs[0]);await cell(s);
      ensure(s.inputKinds.join(",")==="username,click,password","input_route_sequence");
      ensure(load(s,PW)===null&&s.clearEvents.length===1,"password_dispatch_consumes_once");
      finalOutcome(s,null,false);break;
    }
    case "missing_password_refuses_without_input":{
      s=makeState();decision(s,"password","p1:1");await cell(s);
      ensure(s.inputKinds.length===0,"no_input_on_refusal");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "unowned_context_refuses_without_operations":{
      s=makeState();const own=load(s,OWN);own.driver_owned=false;s.map.set(OWN,own);
      await cell(s);ensure(s.trace.length===0&&s.outputs.at(-1).proposal_refused===
        "fresh_owned_context_required","unowned_refusal");break;
    }
    case "undeclared_ref_refuses_input":{
      s=makeState({password:true});decision(s,"click","p1:999");await cell(s);
      ensure(s.inputKinds.length===0,"no_input_on_refusal");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "stale_ref_cannot_cross_cells":{
      s=makeState();decision(s,"click","p1:1");await cell(s);
      ensure(!load(s,OWN).fresh_refs.includes("p1:1"),"fresh_ref_replaces_consumed");
      decision(s,"click","p1:1");await cell(s);
      ensure(s.inputKinds.length===1,"single_use_ref");finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "invalid_kind_repeated_cleanup_clears_before_await":{
      s=makeState({password:true});decision(s,"not-declared");await cell(s);
      ensure(s.clearEvents.length===2&&s.stopCount===1,"repeat_clear_no_repeat_await");
      finalOutcome(s,"browser_decision_unconfirmed",true);break;
    }
    case "helper_zero_final_snapshot_then_cleanup":
    case "helper_zero_missing_page_still_fails":
    case "helper_nonzero_final_still_refuses":
    case "helper_zero_other_kind_still_refuses":{
      s=makeState();
      const other=name==="helper_zero_other_kind_still_refuses";
      const nonzero=name==="helper_nonzero_final_still_refuses";
      decision(s,other?"click":"protected_after",other?"p1:1":undefined);
      s.snapshotQueue=[name==="helper_zero_missing_page_still_fails"?"missing":"after"];
      firstActive(s,event({helper_completed:true,exit:nonzero?1:0}));await cell(s);
      const snapshots=s.trace.filter(x=>x.kind==="snapshot").length;
      ensure(s.inputKinds.length===0&&!s.trace.some(x=>x.kind==="navigate"),"read_only_handoff");
      if(nonzero||other){ensure(snapshots===0,"zero_only_declared_final");
        finalOutcome(s,nonzero?"helper_failed":"helper_completed_before_app_checkpoint",true);}
      else {ensure(snapshots===1,"one_final_snapshot");
        if(name==="helper_zero_missing_page_still_fails")finalOutcome(s,"browser_decision_unconfirmed",true);
        else {ensure(load(s,OBS).protected_after_page===true,"final_page_proved");finalOutcome(s,null,true);}}
      break;
    }
    case "helper_zero_prepared_phase_still_refuses":{
      s=makeState({entry:true});decision(s,"protected_after");
      firstActive(s,event({fixture_ready:true,guard_pid:11,server_pid:12,helper_pid:14,
        lab:"/synthetic/lab",helper_completed:true,exit:0}));await cell(s);
      ensure(!s.trace.some(x=>["navigate","snapshot","type","click"].includes(x.kind)),"prepared_phase_refusal");
      finalOutcome(s,"helper_completed_before_app_checkpoint",true);break;
    }
    case "partial_line_drains_before_output_and_next_cell":
    case "partial_exact_cap_is_accepted":{
      s=makeState();decision(s,"snapshot");
      const line=partialEvent(name==="partial_exact_cap_is_accepted"?16384:undefined);
      firstActive(s);firstActive(s,line.slice(0,-1));firstActive(s,line.slice(-1));await cell(s);
      ensure(s.pollCount===3&&load(s,OBS).unexpected_failure_observation?.observed===true,
        "partial_event_processed_before_output");
      ensure([...s.map.keys()].sort().join(",")===[OWN,OBS].sort().join(","),"no_partial_store");
      finalOutcome(s,null,false);
      if(name==="partial_line_drains_before_output_and_next_cell"){
        const previous=s.pollCount;decision(s,"snapshot");await cell(s);
        ensure(s.pollCount-previous===2,"no_partial_carry_next_cell");finalOutcome(s,null,false);
      }break;
    }
    case "partial_over_cap_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s);firstActive(s,partialEvent(16385));
      await cell(s);finalOutcome(s,"controller_observation_invalid",true);break;
    }
    case "partial_joined_controller_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s);firstActive(s,'{"opaque":"'+PARTIAL,{exit:0});
      await cell(s);finalOutcome(s,"controller_completed_before_app_checkpoint",true);break;
    }
    case "partial_unavailable_controller_refuses":{
      s=makeState();decision(s,"snapshot");firstActive(s,'{"opaque":"'+PARTIAL);
      s.queue.push({throw:true});await cell(s);
      finalOutcome(s,"controller_observation_unavailable",false);break;
    }
    case "partial_deadline_prevents_extra_poll":{
      s=makeState();decision(s,"snapshot");firstActive(s);
      firstActive(s,'{"opaque":"'+PARTIAL,{at:840000});await cell(s);
      ensure(s.pollCount===3&&s.trace.filter(x=>x.kind==="controller_poll"&&!x.cleanup).length===2,
        "no_extra_active_poll_at_deadline");
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "partial_completed_late_still_refuses":{
      s=makeState();decision(s,"snapshot");const line=partialEvent();
      firstActive(s);firstActive(s,line.slice(0,-1),{at:839999});
      firstActive(s,line.slice(-1),{at:840000});await cell(s);
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "retained_start_budget_refuses_next_cell":{
      s=makeState();decision(s,"snapshot");await cell(s);const o=load(s,OWN);
      setTime(s,840000);decision(s,"snapshot");const before=s.trace.filter(x=>x.kind==="snapshot").length;
      await cell(s);ensure(load(s,OWN).start_epoch_ms===o.start_epoch_ms,"startup_budget_retained");
      ensure(s.trace.filter(x=>x.kind==="snapshot").length===before,"no_snapshot_after_deadline");
      finalOutcome(s,"browser_active_deadline",true);break;
    }
    case "inclusive_cleanup_deadline_does_not_invent_join":{
      s=makeState();decision(s,"snapshot");setTime(s,900000);await cell(s);
      ensure(load(s,OBS).controller_exit===null,"no_join_credit_at_inclusive_limit");
      finalOutcome(s,"browser_active_deadline",false);break;
    }
    case "first_page_failure_survives_later_helper_error":
    case "missing_absence_proof_prevents_release":
    case "cleanup_exception_is_private":
    case "pending_metadata_command_prevents_release":{
      s=makeState({cleanup_helper_error:name==="first_page_failure_survives_later_helper_error",
        absence_missing:name==="missing_absence_proof_prevents_release",
        driver_exception:name==="cleanup_exception_is_private",
        metadata_pending:name==="pending_metadata_command_prevents_release",password:true});
      decision(s,"protected_after");s.snapshotQueue=["missing"];await cell(s);
      finalOutcome(s,"browser_decision_unconfirmed",
        name!=="missing_absence_proof_prevents_release"&&name!=="pending_metadata_command_prevents_release");
      if(name==="cleanup_exception_is_private")ensure(load(s,OBS).cleanup_errors.includes(
        "driver_operation_unconfirmed"),"driver_failure_retained");
      break;
    }
    case "projection_exact_pairs_and_private_field_omission":{
      s=makeState();projectionDecision(s,"snapshot","heading","Local demo");
      const nodes=PUBLIC_PAIRS.map(([role,name])=>projectionNode(role,name));
      const refs=[
        projectionRef("button","Sign in","p70:1",["click",PROJECTION]),
        projectionRef("button","Allow","p70:2",["click"]),
        projectionRef("button","Continue","p70:3",["click"]),
        projectionRef("textbox","Username","p70:4",["type"]),
        projectionRef("textbox","Password","p70:5",["type"]),
        projectionRef("textbox","Authenticator or recovery code (if enabled)","p70:6",["type","click"]),
        projectionRef("heading","Local demo","p70:7",["click"])
      ];
      nodes.push(projectionNode("textbox",PROJECTION));
      s.snapshotQueue=[projectionSnapshot(nodes,refs,{page:{url:PROJECTION},account:{subject:PROJECTION},
        error:PROJECTION,headers:{authorization:PROJECTION},cookie:PROJECTION,state:PROJECTION})];
      const out=await cell(s);
      projected(s,out,{observed:PUBLIC_PAIRS.map(([role,name])=>({role,name})),refs:[
        {role:"button",name:"Sign in",action:"click",ref:"p70:1"},
        {role:"button",name:"Allow",action:"click",ref:"p70:2"},
        {role:"button",name:"Continue",action:"click",ref:"p70:3"},
        {role:"textbox",name:"Username",action:"type",ref:"p70:4"},
        {role:"textbox",name:"Password",action:"type",ref:"p70:5"}
      ]});noProjectionInput(s);break;
    }
    case "projection_otp_observed_without_action_or_value":{
      s=makeState();projectionDecision(s,"snapshot","textbox","Authenticator or recovery code (if enabled)");
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("textbox","Authenticator or recovery code (if enabled)")],[
        projectionRef("textbox","Authenticator or recovery code (if enabled)","p71:1",["type","click"])])];
      projected(s,await cell(s),{observed:[
        {role:"textbox",name:"Authenticator or recovery code (if enabled)"}],refs:[]});
      noProjectionInput(s);break;
    }
    case "projection_required_consent_allow_roundtrip":
    case "projection_optional_consent_continue_roundtrip":{
      const required=name==="projection_required_consent_allow_roundtrip";
      const heading=required?"Local demo wants to use your riAuth account":"Continue to Local demo?";
      const label=required?"Allow":"Continue",ref=required?"p72:1":"p73:1";
      s=makeState({projection_protocol:true,expected_action_ref:ref});
      projectionDecision(s,"snapshot","heading",heading);
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("heading",heading),projectionNode("button",label)],[
        projectionRef("button",label,ref,["click"])])];
      projected(s,await cell(s),{observed:[{role:"heading",name:heading},{role:"button",name:label}],
        refs:[{role:"button",name:label,action:"click",ref}]});
      noProjectionInput(s);
      projectionDecision(s,"click","textbox","Username",ref);
      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
        projectionRef("textbox","Username","p74:1",["type"])])];
      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
        refs:[{role:"textbox",name:"Username",action:"type",ref:"p74:1"}]});
      ensure(s.inputKinds.join(",")==="click"&&!load(s,OWN).fresh_refs.includes(ref),
        "projection_consent_dispatch_and_freshness");break;
    }
    case "projection_role_and_label_rejections":{
      for(const pair of [["heading","Allow"],["button","Allow "+PROJECTION]]){
        s=makeState();projectionDecision(s,"snapshot","button","Allow");
        s.snapshotQueue=[projectionSnapshot([projectionNode(...pair)],[
          projectionRef(...pair,"p75:1",["click"])])];
        projectionFailed(s,await cell(s),"browser_decision_unconfirmed");
      }break;
    }
    case "projection_incomplete_and_error_rejections":{
      for(const variant of ["incomplete","fixed_error","bad_status","tool_error"]){
        s=makeState();projectionDecision(s,"snapshot","textbox","Username");
        const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
          refs:[{role:"button",name:"Allow",action:"click",ref:"p76:9"}]};s.map.set(OWN,owned);
        const raw=projectionSnapshot([projectionNode("textbox","Username")],[
          projectionRef("textbox","Username","p76:1",["type"])]);
        if(variant==="incomplete")raw.structuredContent.snapshot.complete=false;
        if(variant==="fixed_error")raw.structuredContent.content_refs.push(
          projectionNode("statictext","Local demo could not complete this request."));
        if(variant==="bad_status")raw.structuredContent.status="refused";
        if(variant==="tool_error"){raw.isError=true;raw.error=PROJECTION;}
        s.snapshotQueue=[raw];projectionFailed(s,await cell(s),
          variant==="tool_error"?"browser_tool_refused":"browser_decision_unconfirmed",
          variant!=="tool_error");
      }break;
    }
    case "projection_missing_or_wrong_advertised_action":{
      s=makeState();projectionDecision(s,"snapshot","textbox","Username");
      const missing=projectionRef("textbox","Password","p77:2",["type"]);delete missing.actions;
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("textbox","Username"),projectionNode("textbox","Password"),
        projectionNode("button","Sign in"),projectionNode("button","Allow")],[
        projectionRef("textbox","Username","p77:1",["click"]),missing,
        projectionRef("button","Sign in","p77:3",["type"]),
        projectionRef("button","Allow","p77:4",["click",PROJECTION])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Sign in"},{role:"button",name:"Allow"},
        {role:"textbox",name:"Username"},{role:"textbox",name:"Password"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p77:4"}]});
      noProjectionInput(s);break;
    }
    case "projection_malformed_refs":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        projectionRef("button","Allow","x1:1",["click"]),
        projectionRef("button","Allow","p1:-1",["click"]),
        projectionRef("button","Allow",PROJECTION,["click"]),
        projectionRef("button","Allow",17,["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
      noProjectionInput(s);break;
    }
    case "projection_disjoint_rows_do_not_infer_association":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      const owned=load(s,OWN);owned.public_decision={observed:[{role:"button",name:"Allow"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p78:9"}]};s.map.set(OWN,owned);
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        {ref:"p78:1",actions:["click"],value:PROJECTION},
        projectionRef("button","Unrelated "+PROJECTION,"p78:2",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[]});
      ensure(!load(s,OWN).fresh_refs.includes("p78:9"),"projection_no_cross_snapshot_join");
      noProjectionInput(s);break;
    }
    case "projection_duplicate_actions_preserve_multiplicity":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      const row=projectionRef("button","Allow","p79:1",["click"]);
      s.snapshotQueue=[projectionSnapshot([
        projectionNode("button","Allow"),projectionNode("button","Allow")],[
        row,clone(row),projectionRef("button","Allow","p79:2",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],refs:[
        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
        {role:"button",name:"Allow",action:"click",ref:"p79:1"},
        {role:"button",name:"Allow",action:"click",ref:"p79:2"}]});
      noProjectionInput(s);break;
    }
    case "projection_latest_success_replaces_previous_snapshot":{
      s=makeState();projectionDecision(s,"snapshot","button","Allow");
      s.snapshotQueue=[projectionSnapshot([projectionNode("button","Allow")],[
        projectionRef("button","Allow","p80:1",["click"])])];
      projected(s,await cell(s),{observed:[{role:"button",name:"Allow"}],
        refs:[{role:"button",name:"Allow",action:"click",ref:"p80:1"}]});
      projectionDecision(s,"snapshot","textbox","Username");
      s.snapshotQueue=[projectionSnapshot([projectionNode("textbox","Username")],[
        projectionRef("textbox","Username","p81:1",["type"])])];
      projected(s,await cell(s),{observed:[{role:"textbox",name:"Username"}],
        refs:[{role:"textbox",name:"Username",action:"type",ref:"p81:1"}]});
      ensure(!load(s,OWN).fresh_refs.includes("p80:1"),"projection_latest_ref_only");
      projectionDecision(s,"click","textbox","Username","p80:1");
      await cell(s);finalOutcome(s,"browser_decision_unconfirmed",true);
      ensure(s.inputKinds.length===0,
        "projection_old_ref_refused");break;
    }
    default:throw failure("unknown_case");
  }
  privacy(s.outputs);privacy(s.metadata);privacy(s.trace);
}
async function main(){
  let current=null;
  try{
    clockGuard();bindSource();
    ensure(BINDING.case_plan.length===new Set(BINDING.case_plan.map(x=>x.name)).size,"unique_case_names");
    for(const row of BINDING.case_plan){
      clockGuard();current=row.name;result.attempted.push(row.name);
      await runCase(row.name);result.completed.push(row.name);
      result.completed_groups[row.group]=(result.completed_groups[row.group]||0)+1;
    }
    ensure(result.cells===51,"projection_complete_cell_count");
  }catch(e){
    const code=(e!==null&&(typeof e==="object"||typeof e==="function")&&FAULTS.has(e))?
      FAULTS.get(e):"unexpected";
    result.first_failure={case:current,check:BINDING.check_names.includes(code)?code:"unexpected"};
  }
  result.unreached=BINDING.case_plan.map(x=>x.name).filter(x=>!result.attempted.includes(x));
  result.elapsed_ms=performance.now();
  if(result.elapsed_ms>=LIMIT_MS&&result.first_failure===null)
    result.first_failure={case:null,check:"real_deadline"};
  // Only the closed finite packet is written; no raw exception/trace/source/stub value.
  const output=JSON.stringify(result);
  if(PRIVATE.some(x=>output.includes(x))){
    process.stdout.write(JSON.stringify({schema:result.schema,first_failure:{case:null,check:"privacy_sink"}})+"\n");
    process.exitCode=1;return;
  }
  process.stdout.write(output+"\n");process.exitCode=result.first_failure===null?0:1;
}
await main();
```

Final actual static receipt: complete candidate/baseline/logic/payload archives
re-extracted to their exact declared bytes/hashes; payload reconstruction and
Python assembly AST parsing passed. python3 scripts/check-docs.py exited0 with
Markdown links and build-directory layout checked; git diff --check exited0.
The complete60522-byte48288 prefix is exact, LF/final-LF/trailing-whitespace
checks passed, only this report is changed and there are no new files.
All36 cases,51 planned cells and future VM/controller remain UNEXECUTED.
The corrected-EOF controller source pin/body prerequisite remains open; no
runtime slot was acquired or released.
