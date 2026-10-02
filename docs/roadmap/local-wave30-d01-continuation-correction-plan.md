# D01 continuation boundary correction plan

Date: 2026-10-03, Europe/Vaduz. SOURCE DESIGN ONLY; candidate UNEXECUTED.

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original D01
`a96a1977-3210-4284-8f7d-645793369301`; existing supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`.
Reservation: `wave30_D01_continuation_boundary_correction_plan`.
The sole write is this new report, on the existing own branch from clean parent
`4c4b06160e4ab164fa27c40035fba3a7974bb215`. No alignment or merge.

This report proposes only the F2/F3/F4 continuation-cell corrections identified
by the independent immutable review. The complete 485-line candidate below
retains the accepted phase fix and changes four exact source spans. Reversing
those spans, or independently reversing all four unified-diff hunks, reconstructs
the entire 31581-byte phase-corrected cell. A separate structural AST inverse
also matches the entire phase-corrected AST. This is a source-preservation result,
not execution of the cell, its collectors, controller, helper or a journey.
Root must review the immutable design before any separately reserved memory
envelope or real fixture. All such runtime remains HELD; no slot was acquired.

## Exact immutable inputs and read scope

I read the full 378-line independent findings report at
`309ee22e5e813fb3bea31a1f836b9ebb131dac0a:docs/roadmap/local-wave30-d01-controller-boundary-independent-review.md`.
I read the complete phase-correction appendix, including the complete 467-line
cell and its archived static proof, at
`c5d4d173857d040947a8cb3780c47c7141f98336:docs/roadmap/local-wave30-d01-user-browser-review.md`.
The baseline for these changes is that cell, not an older checked-out source
or another worker's mutable candidate.

The original cell and full 225-line controller command, plus the 37-line
preparation boundary, were extracted from
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f:docs/roadmap/local-wave30-d01-user-browser-review.md`.
The original cell equals the fully read phase-corrected cell with exactly the
120-byte phase block removed. I read the full controller command and preparation
boundary for the event/newline/ownership context. All five embedded Python
collector literals were retained and parsed, without invocation.

| Immutable source | Bytes / lines | SHA256 |
| --- | --- | --- |
| Independent 309 findings report | 23341 / 378 | `68c5f02de8d16aec39adaf582f17f7de4595267354260fb45e467378e15b24c4` |
| Entire 2e29 report | 635391 / 9093 | `c3d7d338676cc1156a9958253c2b51eab823eb25b7478c9ec1b0d4fba37c91a8` |
| Entire c5 report | 678273 / 9769 | `cfb1ded0643943622720322cba7d59dc0ef44fd09977c15cf97951dfa93ef910` |
| Original 2e29 cell, document 7987–8449 | 31461 / 463 | `eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a` |
| Phase-corrected c5 cell, document 9142–9608 | 31581 / 467 | `d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d` |
| Controller command, 2e29 document 7698–7922 | 17326 / 225 | `5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0` |
| Preparation boundary, 2e29 document 8776–8812 | 2217 / 37 | `1fb412fbf551a3f76595fca551adb2828775cdd3dfa1801e798426557a6e1355` |
| Proposed F2/F3/F4 cell | 32502 / 485 | `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8` |
| Complete four-hunk diff | 2243 / 52 | `395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144` |

The c5 report begins with all 635391 original 2e29 bytes, including every old
source pin and historical failure. The controller command's canonical hash
excludes the single fence-delimiter newline after its final `PY`; the cell's
final newline is part of its canonical bytes. Neither old report is edited.

At fixed public `35c3fd3007c52d8142c7bee1d69aee127cc42a95`, I hash-checked the
entire 757-line `scripts/d01-confidential-browser-demo.py` (35749 bytes,
`75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`) and
read lines 532–682: cookie parsing, protected-page response, callback redirect,
and main's completion predicates. This is selected-body review plus a whole-file
hash check, not a claim to have reread all helper bodies or run them.
The authenticated `/protected` response sets the accepted-cookie check and
`demo.done` before main checks the full security predicates and finishes.
Consequently helper exit zero can precede the caller's final read-only snapshot
without that zero exit itself proving any snapshot or journey.

## F2: consume the password only on its password dispatch; clear at cleanup entry

In the c5 cell, `continuation` binds the same session/target/tab/ref, consumes
the accepted snapshot ref, and validates the private password before the
password operation. Its unconditional clear after every input also runs for
click and username. A preloaded password can therefore be erased before the
later password decision. This is a deterministic source ordering defect,
not proof that a historical caller supplied or lost a password.

The post-input clear becomes exactly
`if(spec.kind==="password")store("d01_fresh_password_input",null);`.
It remains after the awaited password dispatch and before its follow-up
snapshot. Click/username/snapshot operations do not consume the password merely
by progressing. If a refusal, exception or deadline prevents or fails a
password dispatch, the existing cleanup path performs the clear instead.
The accepted `confirmed`/`unverifiable` dispatch predicate is unchanged;
this report does not upgrade that predicate into proof of actual insertion.

The first statement of owned `cleanup` clears that same store key, before the
idempotency return and before any await. Thus both first and repeated owned
cleanup requests clear the ephemeral value; stop/clock persistence still
precedes all Driver awaits and retains its existing order. This is an in-memory
store clear, not a new password-file read, transfer, input schema or secret
adapter. The initial ownership/context refusal remains unchanged. No missing
preparation or partial-ownership cleanup contract is guessed or broadened.

## F3: helper zero may reach only the declared final read-only snapshot

The controller prints its newline-terminated helper-completed event once the
helper finishes. The c5 observer unconditionally refuses helper zero when
`protected_after_page` is not yet true. The final continuation polls that
observer before calling the read-only snapshot, so a valid queued zero event
can prevent the very snapshot required to prove the protected page.

The only relaxation is the already-declared conjunction
`own.phase==="browser_decision" && own.next_decision?.kind==="protected_after"`.
It applies only to the zero branch, before cleanup, while page proof is absent.
The nonzero branch, every other phase/decision, controller completion, malformed
event, ownership mismatch, and existing first failure retain their refusals.

The unchanged continuation validates its declared kind before that poll,
then calls only `get_browser_state(snapshotArgs)` with the original bound
session/target/tab. The unchanged `page(...,"protected_after")` still requires
status ok, a complete snapshot, the exact protected heading and exact public
authenticated text, without the error label. Missing proof latches
`browser_decision_unconfirmed` and invokes cleanup. Success proceeds to the
existing stop/cleanup, not another RP navigation or HTTP request. A zero exit
without that snapshot earns no application/journey credit. No factor,
credential, callback, token, cookie, source, authorization or transport policy
is changed.

## F4: drain a partial line within this invocation before final output

The observer owns a local `controllerBuffer`, admits at most 16384 characters,
and parses only newline-terminated JSON events. Its numeric result projection
is retained before comparisons. A partial suffix currently survives within
one invocation but is silently lost when that invocation returns; neither
`own` nor the observation record safely stores unvalidated raw output.

The proposed drain is inside the existing top-level try, immediately after
entry/continuation and before catch/final output. It runs only while a partial
suffix remains and no first failure exists. It awaits only the unchanged
`pollController`, which names exactly `own.exec_session` and retains the
numeric receipt before applying event comparisons. There is no new session,
tool, reader, command, SQL/HTTP request, marker or raw-output sink.

Before each extra poll and after its return, the drain compares wall time to
the existing `own.start_epoch_ms+840000` active boundary. At or after that
boundary it latches the existing fixed `browser_active_deadline` label.
A joined/unavailable controller with an unfinished line latches the existing
`controller_observation_invalid` label; existing event/transport failures
keep their original first label. The unchanged first-failure latch cannot be
overwritten. A successful return with empty partial buffer needs no extra
poll. After any drain failure the existing owned cleanup runs before output.
Its idempotent guard prevents a second cleanup sequence.

This is a finite active-budget drain under the same wall-clock/deadline model
as the archived cell. It adds no retry interval, new budget, deadline reset or
stored partial string. Essential join/absence cleanup keeps the existing
START+900000 inclusive budget and late-cleanup behavior, exactly. An awaited
tool can still return late or lack cancellation; a backwards/nonprogressing
wall clock is not fixed by this scoped design. Neither a hard call-timeout
guarantee nor whole-cleanup-within60 is claimed. Those existing timing limits
are not erased by a successful static parse.

Raw partial text exists only in the lexical buffer during this invocation.
On failure, existing cleanup clears that buffer. It is never persisted in
`own`, diagnostic metadata or public output, and no new log/printing branch
is introduced. The controller's fixed JSON print bodies are unchanged.

## Exact diff — DESIGN ONLY

All coordinates below refer to the complete immutable c5 cell. The only
affected functions are `observeController`, `cleanup` and `continuation`;
the fourth hunk inserts the local drain into the existing top-level try.

```diff
--- immutable-c5d4d173-cell-d7278780
+++ DESIGN-only-F2-F3-F4
@@ -139,7 +139,9 @@
       observation.helper_exit=typeof event.exit==="number"?event.exit:null;
       retain();
       if(event.exit!==0)latch("helper_failed",receivedWall);
-      else if(!cleanupStarted&&record.protected_after_page!==true)
+      else if(!cleanupStarted&&record.protected_after_page!==true&&
+              !(own.phase==="browser_decision"&&
+                own.next_decision?.kind==="protected_after"))
         latch("helper_completed_before_app_checkpoint",receivedWall);
     }
     if(event.fixture_finished===true) {
@@ -189,6 +191,7 @@
   retain();
 }
 async function cleanup() {
+  store("d01_fresh_password_input",null); // Clear before any cleanup await.
   if(cleanupStarted)return;
   cleanupStarted=true;record.cleanup_started=true;retain();
   try {
@@ -415,7 +418,7 @@
       const s=r?.structuredContent;
       return r?.isError!==true&&["confirmed","unverifiable"].includes(s?.effect);
     }); // Dispatch alone never earns an application outcome or journey credit.
-    store("d01_fresh_password_input",null);
+    if(spec.kind==="password")store("d01_fresh_password_input",null);
     if(record.first_failure===null)
       await checked("browser_snapshot",
         ()=>tools.mcp__cua_driver__get_browser_state(snapshotArgs),
@@ -442,6 +445,21 @@
 }
 try {
   if(own.phase==="prepared_entry")await entry();else await continuation();
+  // Do not carry unvalidated partial controller text across this cell boundary.
+  while(controllerBuffer.length>0&&record.first_failure===null) {
+    const beforePollWall=Date.now();
+    if(beforePollWall>=own.start_epoch_ms+840000) {
+      latch("browser_active_deadline",beforePollWall);break;
+    }
+    if(controllerJoined||!controllerPollAvailable) {
+      latch("controller_observation_invalid",beforePollWall);break;
+    }
+    await pollController(); // Same owned session; observer retains receipt first.
+    const afterPollWall=Date.now();
+    if(afterPollWall>=own.start_epoch_ms+840000)
+      latch("browser_active_deadline",afterPollWall);
+  }
+  if(record.first_failure!==null)await cleanup();
 } catch {
   latch("composed_decision_exception",Date.now());
   await cleanup();
```

## Complete candidate cell — NOT EXECUTED

32502 UTF-8 bytes / 485 lines; SHA256
`7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8`.
The final newline before the closing fence is part of this identity.
This archive is source for review only, not runtime authorization.

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

## Byte/AST inverse and protected source proof

Four old/new string spans each match exactly once. Replacing only those four
new spans in reverse order restores every c5 cell byte and the `d7278780`
hash. An independent unified-diff inverse checks every context/addition
against candidate line coordinates, reverses all four hunks, and also restores
the entire c5 cell. Removing only its separately identified 120-byte phase
block then restores the complete 2e29 `eff0cdad` cell. No baseline source is
constructed from another worker's current files.

An independent Acorn AST check parses original, phase-corrected and proposed
cells as modules without evaluating them. It normalizes only source positions
and labels existing BigInt literals by decimal text. It removes the first
cleanup clear, restores exactly the old post-input clear and old helper-zero
conditional, and removes exactly the drain while plus its failure-cleanup
statement from the top-level try. The entire resulting AST equals c5.
The phase block's position and entire entry function remain exact; removing
that one block independently restores the original 2e29 AST.

| AST identity | SHA256 of normalized AST JSON |
| --- | --- |
| Original 2e29 | `103b85f3ea8e2b805af3afb031b56ac5378595130f9f8bac38d43df7695d472a` |
| Phase-corrected c5 | `28a72ac4b2b27a5c6f6aeee63883557580e4b63e9de097c9339e9dd9fd33d750` |
| Proposed candidate | `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74` |
| Four-edit structural inverse | `28a72ac4b2b27a5c6f6aeee63883557580e4b63e9de097c9339e9dd9fd33d750` |

Ten of the thirteen complete function source spans are byte-identical.
The three changes listed below are precisely the declared F2/F3 spans; F4
changes no function. All other top-level source is exact outside that drain.

| Function | Preservation | c5 complete body SHA256 | Candidate complete body SHA256 |
| --- | --- | --- | --- |
| finiteObservation | Exact | `b57854dabbc985407451ab0f90272a0250218156ea298750b5550951193b19c1` | `b57854dabbc985407451ab0f90272a0250218156ea298750b5550951193b19c1` |
| diagnostic | Exact | `08fb60069a6523d07783754388cf2479d3a2895a1607ec87fb80101b136d0af1` | `08fb60069a6523d07783754388cf2479d3a2895a1607ec87fb80101b136d0af1` |
| latch | Exact | `6cc3aae70510550f21304ceb79587d03c800cb459d50c1bd714ea77f4541199c` | `6cc3aae70510550f21304ceb79587d03c800cb459d50c1bd714ea77f4541199c` |
| observeController | F2/F3 only | `702b8e20d30f9d9044293a951a52afe63bb6e86847d5e0a2c65da54c88588b5d` | `1e8968dd48f70ab7c6b88fbb250e1e4ba57879604a596a8d901a1f0bd635cb16` |
| pollController | Exact | `6dfcfa6d89bfc4345ca77a8304bd8390353fb8a3e80285c9c66e7187ff2d7d4c` | `6dfcfa6d89bfc4345ca77a8304bd8390353fb8a3e80285c9c66e7187ff2d7d4c` |
| timedDriver | Exact | `f7c305d432a3a11885b71035e771ca0567520165fbe697f74f2d65a2d981bd0b` | `f7c305d432a3a11885b71035e771ca0567520165fbe697f74f2d65a2d981bd0b` |
| cleanup | F2/F3 only | `847d37e4b772570ddb5822ee2c22ebe21cba9f0a4c34adb04f44e5826ab27537` | `0d8dc617795d72e24519b956f324cf08145838e157840418f107e8436457ffca` |
| checked | Exact | `df302d63be0451bcb392a0d6ab733f7d9d552a5605a82e99375bb6ac8b8cc1a9` | `df302d63be0451bcb392a0d6ab733f7d9d552a5605a82e99375bb6ac8b8cc1a9` |
| page | Exact | `d4aa96f964c11de6a2d8a15dbfe965d69edf40a0e3bf38d9c9cc614dcdafd88c` | `d4aa96f964c11de6a2d8a15dbfe965d69edf40a0e3bf38d9c9cc614dcdafd88c` |
| navigateAndSnapshot | Exact | `0f2337a9f2c8ba20c34be23f1249d4352f21267ea8f45b475a2ce72d60b11a65` | `0f2337a9f2c8ba20c34be23f1249d4352f21267ea8f45b475a2ce72d60b11a65` |
| entry | Exact | `6b6350f4abd79a799c09fa4c1326a45fcdf824818d221aba0e790608e0902118` | `6b6350f4abd79a799c09fa4c1326a45fcdf824818d221aba0e790608e0902118` |
| continuation | F2/F3 only | `905a6e1a938488a17ad232aaa21f895c0fcc45b7c554756f05245e2e4e0573b0` | `04365b46726732d62762a351c82df05cb7354b2aefb65600674898068410c0cb` |
| publicPredicate | Exact | `5f4f55c95ae0d062c85892ba92a09ae8263d981f63fe8fd4bcc81da1c29778a5` | `5f4f55c95ae0d062c85892ba92a09ae8263d981f63fe8fd4bcc81da1c29778a5` |

The five collector declarations are byte-identical. Their unescaped source
values have these independent hashes, and each parses with Python AST only:

| Literal | Bytes | Source SHA256 |
| --- | --- | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

The controller command and preparation boundary are not modified or copied
into a new executable file. Python AST parses the controller body as source
only. The controller/helper/source pins, disk sampling and 8.5 GiB floor,
0700/0600 exclusive evidence, exact PIDs/ports/session/target/tab/ref,
single-controller gates, marker/stop protocol, one-shot helper creation,
600/180/60/30/5-second limits, 262144-byte evidence cap, 16384-character
controller cap, diagnostic allowlists and receipt-before-comparison order are
unchanged. Credential-once/receipt-secret, route-specific headers, PAM, held
Group, workflow/source authorities and data writers receive no edits.

The controller still pins server
`7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606`,
maintenance
`86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95`,
client
`bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf`,
verifier `9cefe7a56425bb73c17753e8766d92320b77da3b` /
`f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`,
and provider
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`.
These remain archived prospective input identities, not newly executed
binary/provider checks or current artifact compatibility certification.

## Complete static AST checker archive

4514 bytes; SHA256
`91160d966e2e9078e9b9406b31a8f23a3827b174fb66a676ea5273d2e4be2007`.
The stdin envelope contains only the three public source strings, exact drain
source and phase source. This checker imports the already installed bundled
Acorn parser, parses source and compares AST/data. It does not import a helper,
create a VM, evaluate the cell or call any tool referenced by the cell.
The candidate archive must not be substituted into an executable cell.

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
const baseRaw=parse(input.base),candidateRaw=parse(input.candidate),originalRaw=parse(input.original);
const base=normalize(baseRaw),candidate=normalize(candidateRaw);
function replaceUnique(root,from,to) {
  let count=0;
  function visit(v) {
    if(key(v)===key(from)){count++;return structuredClone(normalize(to));}
    if(Array.isArray(v))return v.map(visit);
    if(v&&typeof v==="object")return Object.fromEntries(Object.entries(v).map(([k,x])=>[k,visit(x)]));
    return v;
  }
  const result=visit(root);
  if(count!==1)throw Error("non_unique_ast_edit");
  return result;
}
let restored=structuredClone(candidate);
const cleanup=restored.body.find(n=>n.type==="FunctionDeclaration"&&n.id.name==="cleanup");
const clear=parse('store("d01_fresh_password_input",null);').body[0];
if(key(cleanup.body.body[0])!==key(clear))throw Error("cleanup_first_statement");
cleanup.body.body.shift();
restored=replaceUnique(restored,
  parse('if(spec.kind==="password")store("d01_fresh_password_input",null);').body[0],
  parse('store("d01_fresh_password_input",null);').body[0]);
const oldHelper='if(!cleanupStarted&&record.protected_after_page!==true)\n  latch("helper_completed_before_app_checkpoint",receivedWall);';
const newHelper='if(!cleanupStarted&&record.protected_after_page!==true&&\n  !(own.phase==="browser_decision"&&own.next_decision?.kind==="protected_after"))\n  latch("helper_completed_before_app_checkpoint",receivedWall);';
restored=replaceUnique(restored,parse(newHelper).body[0],parse(oldHelper).body[0]);
const rootTry=restored.body.find(n=>n.type==="TryStatement");
const expectedDrain=normalize(parse(input.drain)).body;
if(expectedDrain.length!==2||rootTry.block.body.length!==3||
   key(rootTry.block.body.slice(1))!==key(expectedDrain))throw Error("drain_position_or_body");
rootTry.block.body.splice(1,2);
if(key(restored)!==key(base))throw Error("whole_ast_inverse");
const functions=baseRaw.body.filter(n=>n.type==="FunctionDeclaration");
const functionHashes=functions.map(n=>{
  const c=candidateRaw.body.find(x=>x.type==="FunctionDeclaration"&&x.id.name===n.id.name);
  return {name:n.id.name,base_sha256:hash(input.base.slice(n.start,n.end)),
    candidate_sha256:hash(input.candidate.slice(c.start,c.end)),
    bytes_equal:input.base.slice(n.start,n.end)===input.candidate.slice(c.start,c.end)};
});
if(functionHashes.filter(n=>n.bytes_equal).length!==10)throw Error("function_scope");
const constantNames=["CLOCK","STOP","READBACK","MARKER","PERSIST"];
const constants=constantNames.map(name=>{
  const get=ast=>ast.body.find(n=>n.type==="VariableDeclaration"&&n.declarations.some(d=>d.id.name===name));
  const b=get(baseRaw),c=get(candidateRaw);
  if(input.base.slice(b.start,b.end)!==input.candidate.slice(c.start,c.end))throw Error("collector_changed");
  return {name,source_bytes:Buffer.byteLength(b.declarations[0].init.value),
    source_sha256:hash(b.declarations[0].init.value),source:b.declarations[0].init.value};
});
const phase=normalize(parse(input.phase)).body[0];
const entry=base.body.find(n=>n.type==="FunctionDeclaration"&&n.id.name==="entry");
if(key(entry.body.body.at(-2))!==key(phase))throw Error("phase_position");
const baseWithoutPhase=structuredClone(base);
baseWithoutPhase.body.find(n=>n.type==="FunctionDeclaration"&&n.id.name==="entry").body.body.splice(-2,1);
if(key(baseWithoutPhase)!==key(normalize(originalRaw)))throw Error("original_phase_ast");
console.log(JSON.stringify({parser_only:true,candidate_evaluated:false,whole_ast_inverse:true,
  base_normalized_ast_sha256:hash(key(base)),restored_normalized_ast_sha256:hash(key(restored)),
  candidate_normalized_ast_sha256:hash(key(candidate)),
  original_normalized_ast_sha256:hash(key(originalRaw)),phase_retained:true,
  four_exact_ast_edits:true,top_level_functions:functionHashes.length,functionHashes,constants}));
```

## Actual checks, historical evidence and remaining release boundaries

Actual static checks performed here: immutable Git reads and SHA256/size/line
checks; exact 2e29-prefix identity in c5; four unique-span byte inverse;
independent four-hunk diff inverse; Acorn syntax/structural AST inverse and
function/literal hashes (exit 0); Python AST parsing of the five collector
values and controller body (no imports/invocation). No candidate/case/cell,
VM, stub, collector, controller, bootstrap or helper is executed by these checks.
No native/CLI/provider/HTTP/Driver/browser/Cargo or service is invoked.

Static preparation corrections are retained explicitly. First archive
extraction exited 1 because it included the delimiter newline after the
controller's final PY; excluding that one delimiter byte recovered the
canonical 17326-byte command/hash, with the cell bytes unchanged. The first
AST checker exited 0 and proved its inverse, but aliased its candidate AST
while removing the cleanup statement, so its auxiliary candidate-AST hash
was not an untouched-candidate identity. I corrected the checker to deep-copy
that AST before inversion; the repeated parser-only check exited 0 and yielded
the exact hashes archived above. No candidate bytes changed, no candidate ran,
and neither issue is a helper/fixture/product failure.

An initial final archive checker confirmed all three fence bodies and their
hashes, then exited 1 because its own script contained a stray `PY` identifier.
Removing that checker-only line yielded the final exit-0 check. Its measured
diff line count also corrected this report's draft table from 54 to 52; the
2243 diff bytes/hash and all candidate bytes stayed unchanged.

I also read the two complete published public memory receipts at fixed
`35c3fd3007c52d8142c7bee1d69aee127cc42a95`:
`docs/roadmap/evidence/wave30-d01-descriptor-memory-0ec651d.json`
(5638 bytes / 155 lines, SHA256
`5df541995dd1f74bff9c91104400140040d986a98a9f13f8688fbcb82cc4782a`)
and `docs/roadmap/evidence/wave30-d01-cleanup-memory-2bf9ebc.json`
(10004 bytes / 368 lines, SHA256
`f2832e4397e69e59bb86680d975f46a06ca1dbdb7c830d4f949823be11bc989a`).
Root's descriptor run completed 128/128 (50 observer/78 legacy), exit 0/reaped,
one spawn, child 0.101654 seconds and controller 0.201888 seconds. It covers
selected helper definitions and controlled Python memory/frame/traceback
objects; it does not execute this browser cell, controller or preparation.
Root's separate old-cell run completed 18/18, exit 0/reaped, 0.15420270897448063
seconds, with 286 stub calls. Its source cell is the old 284-line
`0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e`.
The within-cell partial-JSON case is not evidence for this changed
multi-invocation boundary. I did not rerun either memory payload.

All historical capacity/preparation/provider/request/Authorization failures
and source/static correction limits remain intact in the original reports.
The earlier 61/76 failed verification, historical 78-case pass, later 128-case
pass, old 18-case pass and historical c88 fixture are separate outcomes.
Historical c88 unexpected failure, zero Authorization count, protected-before
403 and no journey remain unchanged. The actual Authorization sender/cause,
lost provider values, true first cleanup event and whole60 remain UNKNOWN.
The roughly 119-second final readback is not an earlier cleanup anchor.
F2/F3/F4 findings and this changed candidate are not retrospective attribution.

Exact preparation, private-input receipt/transfer and partial-ownership
cleanup remain the separately owned Sol4 design. I do not supply its adapter,
assume MCP receipt fields, contact its author or change its report.
Root composes and reviews the designs, retains source/runtime release and
original D01 interpretation/status/integration. There is no D01 completion
recommendation from this unexecuted correction. Cua.ai Driver MCP remains
the only permitted future desktop provider, with descriptions/state inspected
first; no desktop tool was used here.

The own-branch global docs checker before this report exited 1 solely for
five existing private build-output directory names: `target-wave29-source`,
`target-wave28-scim`, `target-wave28-portal`, `target-wave28`,
`target-wave27`. It reported no Markdown-link failure. No checker, directory,
cache, older report or product file is changed to hide that result.
Final report archive/fence/hash/scope/docs/whitespace checks and the sole
report-only commit are recorded below and in the handoff.

Final actual static checks: all three closed source fences match the exact
2243-byte diff, 32502-byte candidate and 4514-byte parser checker; each hash
matches the table/archive above. The whole-cell four-span inverse and phase
inverse, complete old-report prefix, all referenced commit-object resolutions,
and direct trailing-whitespace scan passed (exit 0). The post-write docs
checker exited 1 with byte-identical output to the pre-write five-directory
baseline and no link error. The unstaged no-index whitespace invocation had
empty diagnostics and exit 1 for the new-file difference; after staging,
`git diff --cached --check` exited 0. The staged scope is exactly this one new
report. No archived code was evaluated, no old source/evidence was changed,
and no runtime outcome or status was inferred. Final commit/hash and clean
worktree proof are supplied in the handoff.
