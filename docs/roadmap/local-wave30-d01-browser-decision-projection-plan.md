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
