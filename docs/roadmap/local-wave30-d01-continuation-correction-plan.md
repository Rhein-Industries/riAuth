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

## Dated source-only design: continuation memory envelope, 2026-10-03

Reservation `wave30_D01_continuation_memory_envelope_design`, original D01
`a96a1977-3210-4284-8f7d-645793369301`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing WT42
`42bb51c6-c198-4adb-bd92-0a5222853231`. APPEND ONLY.
All 61754 bytes of `9f3a4a372b53fc6d73d5df45ad1a8d217ad60879` above are preserved,
SHA256 `fd32648eac52c94add32138f4721a96e16f56ce4d6858964363516160410a1db`.
No old source, candidate, diff, historical outcome or failure is replaced.
Root's reported exact staging `0feb205` is context, not a new execution here.

This is a complete prospective ONE installed-Node memory payload and bounded
controller/assembly. It is NOT EXECUTED. No VM, cell, stub, case, controller,
collector, bootstrap, helper, native product, browser, Driver, HTTP, service,
provider or Cargo execution occurs in this phase. Installed Node is used only
as an Acorn source parser in the static checks. No runtime slot, evidence
directory, synthetic process group or fixture is allocated.

The future harness binds and runs the exact COMPLETE 32502-byte candidate
in an isolated VM async-function wrapper. It never substitutes selected
functions or edits the candidate per case. Both full source strings and the
four-hunk diff are carried in the payload; before any VM construction, the
payload checks their lengths/hashes, reverses the four unique spans and
independently reverses every diff line to reconstruct the complete c5 cell.
The controller separately checks the complete assembled payload identity.

| Archived unit | UTF-8 bytes / lines | SHA256 |
| --- | --- | --- |
| Exact candidate, retained in prefix and encoded in payload | 32502 / 485 | `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8` |
| Complete c5 baseline encoded in payload | 31581 / 467 | `d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d` |
| Exact readable diff in prefix and encoded in payload | 2243 / 52 | `395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144` |
| Readable harness logic suffix of payload | 26197 / 476 | `639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054` |
| Complete installed-Node payload below | 132725 / 723 | `46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866` |
| Complete controller, including payload assembly below | 209948 / 2124 | `e3bde0116783eccd01bc6c9c3183c06ade99af21d95800fa47327de9c1ca2207` |

The source data are ASCII inside both programs; the complete raw source and
final newlines are hashed. Encoded source is base64 DATA, not an extra
executable case or an authority/credential. The full candidate and exact diff
remain readable in the preserved prefix. The baseline is exactly the complete
c5 archive at `c5d4d173857d040947a8cb3780c47c7141f98336`, not a stale checked-out
source. The wrapper's body AST statically equals that candidate's complete
module statement array, ignoring source positions only. All thirteen candidate
functions, F1 phase correction, F2/F3/F4 changes, helper/controller pins and
all five embedded collector literals remain exact.

### Meaningful case inventory — all UNEXECUTED

The 25 names below are extracted from the harness's SwitchCase AST and matched
one-to-one to its fixed plan. These are declared scenarios, not 25 passes.
Their source groups are phase_binding 5, secret_lifetime 3, helper_handoff 5,
partial_framing 7 and cleanup_latch 5. The future packet derives attempted,
completed, unreached and completed_groups from actual progress through those
cases. A first unexpected failure stops all later cases; no retry, source
correction or expectation adjustment is authorized.

| Declared case | Group | Observable condition checked by the full-cell consumer |
| --- | --- | --- |
| `phase_entry_then_continuation` | phase_binding | Ready proof is retained after its numeric receipt; the successful entry transitions to browser_decision and a second full cell uses the original start. |
| `password_survives_until_password_dispatch` | secret_lifetime | Username, click and snapshot in separate cells preserve the synthetic password; only the later password dispatch consumes it, with the correct bound arguments. |
| `missing_password_refuses_without_input` | secret_lifetime | Missing password yields no input operation, a fixed refusal and essential cleanup. |
| `unowned_context_refuses_without_operations` | phase_binding | An unowned context returns its original refusal without any tool operation; no partial-preparation cleanup is invented. |
| `undeclared_ref_refuses_input` | phase_binding | A ref absent from the accepted snapshot cannot dispatch input. |
| `stale_ref_cannot_cross_cells` | phase_binding | The previous consumed ref cannot be reused in the next cell after a fresh snapshot. |
| `invalid_kind_repeated_cleanup_clears_before_await` | secret_lifetime | Natural repeated cleanup clears the password twice but performs the stop protocol only once; the STOP stub observes null before its first await. |
| `helper_zero_final_snapshot_then_cleanup` | helper_handoff | Queued helper zero permits exactly the declared bound read-only final snapshot, then essential cleanup; no navigation/input or journey credit. |
| `helper_zero_missing_page_still_fails` | helper_handoff | Helper zero without the protected-page predicates still refuses and cleans up. |
| `helper_nonzero_final_still_refuses` | helper_handoff | Nonzero helper exit refuses before the final snapshot. |
| `helper_zero_other_kind_still_refuses` | helper_handoff | Helper zero with a click decision and no page proof refuses before any browser action. |
| `helper_zero_prepared_phase_still_refuses` | helper_handoff | Helper zero during prepared_entry refuses even if a final decision was declared. |
| `partial_line_drains_before_output_and_next_cell` | partial_framing | Post-decision partial data must be completed by the same cell's extra poll and project a diagnostic before output; a second full cell reuses only the context store, with no partial carry. |
| `partial_exact_cap_is_accepted` | partial_framing | A two-chunk 16384-character newline event is admitted at the exact original cap, with private unknown fields discarded. |
| `partial_over_cap_refuses` | partial_framing | The same event at 16385 characters refuses with the fixed invalid-observation label. |
| `partial_joined_controller_refuses` | partial_framing | Partial text accompanying controller completion retains the original completion refusal. |
| `partial_unavailable_controller_refuses` | partial_framing | A private controller exception while a partial line remains yields fixed unavailability; absence cannot invent a join/release. |
| `partial_deadline_prevents_extra_poll` | partial_framing | A partial event at START+840000 gets no further active poll; essential cleanup join is separately allowed. |
| `partial_completed_late_still_refuses` | partial_framing | A newline arriving exactly at START+840000 does not earn a successful return after the drain. |
| `retained_start_budget_refuses_next_cell` | phase_binding | A next-cell snapshot at the retained active deadline refuses; the start epoch cannot be reset. |
| `inclusive_cleanup_deadline_does_not_invent_join` | cleanup_latch | At START+900000, cleanup cannot fabricate the controller join; the output withholds release. |
| `first_page_failure_survives_later_helper_error` | cleanup_latch | A page refusal remains the first failure after a later nonzero helper event during cleanup. |
| `missing_absence_proof_prevents_release` | cleanup_latch | A refused page plus incomplete fixed absence proof withholds release independently of the refusal. |
| `cleanup_exception_is_private` | cleanup_latch | A private Driver-stub error is retained as a fixed cleanup error; independent complete synthetic absence may still establish resource release, without journey credit. |
| `pending_metadata_command_prevents_release` | cleanup_latch | A pending metadata collector receipt withholds release even when other synthetic absence fields are true. |

The password-lifetime case crosses four independent VM cells; the phase,
stale-ref, partial-drain and retained-budget cases also cross independent
cells. Every cell reloads the same serializable synthetic store and gets a
fresh VM context. Ref decisions use the previous accepted snapshot's fresh
ref; the stale-ref case deliberately supplies its consumed predecessor.
No raw partial buffer is carried in that store. The partial diagnostic must
be processed before the first cell returns, and the next cell receives only
new controller output. The joined/unavailable cases preserve their existing
labels rather than inventing a new successful drain.

These are behavior contrasts with the reported defects: the old unconditional
clear cannot satisfy the non-password lifetime sequence; the old helper-zero
refusal cannot reach the valid final snapshot; the old boundary cannot project
the post-decision partial event before output. The phase and other negative
cases protect the existing guards. No pass is inferred from literal call
counts alone or from agreement with a copied implementation function.
The guard-comparison counter includes identity/privacy/routing checks; it is
not advertised as a separate number of security cases. Case completion is
recorded only after its meaningful assertions return.

### Isolated synthetic transport, privacy and ownership

The VM receives only stubbed tools, store/load/text/exit and a trusted synthetic
Date.now. The real monotonic/performance budget remains outside that synthetic
clock. Code generation from strings and wasm inside the VM is disabled.
The wrapper contains the full reviewed source. This is a closed-source memory
consumer exercise, not a general security sandbox for arbitrary programs.

All tool names present in the candidate resolve to finite local stubs. The
Python-looking exec_command strings are strictly decoded as DATA and matched
against the five exact collector literal values/hashes. They are NEVER sent
to a shell/Python/tool or evaluated. The stubs return controlled public shapes
for marker, clock, stop, join, fixed absence and exclusive metadata-write
receipts; none reads a lab/password file, signals an application PID, makes a
socket request, verifies a token, starts a helper or calls RiWork/Driver.
Synthetic PIDs 11–25 and the synthetic session/target/tab are fixture data,
not real ownership receipts. The only future actual child/process group is
the installed Node owned by the memory controller.

The synthetic password and the raw partial/exception sentinels are fixed,
public fixture strings, never real credentials. Password text is compared only
inside the internal browser_type stub to verify correct routing, then discarded
from its trace; only kind and password_matches booleans survive. The private
input store is intentionally internal while the password is pending. All
public text outputs, observation/owned records, persisted metadata and stub
traces are scanned for all three sentinels. The source password clear is
observed before the STOP stub's first await and again after that await.
Natural repeated cleanup produces two clears and only one essential stop.

Trace data contain finite operation names, booleans and counters, not raw
commands/results/partial strings or exception text. Exceptions are projected
through fixed check codes; the harness never reads an arbitrary message,
stack, repr or string conversion. Source-level host assertion failures are
remembered even if the candidate catches them; after the current bounded
cell/essential synthetic cleanup they stop the harness before any later cell
or case. Deliberately injected plain private exceptions remain expected input
to the candidate's unchanged fixed-label catches.

Release proof is tested separately from first refusal and journey. Synthetic
absence can prove release despite a retained Driver error; missing absence,
unjoined controller or pending metadata command must withhold it. Whole60 and
journey remain false/unproved in all outputs. No collector validator, native
PID/listener absence, true cleanup anchor, actual browser role/ref association,
HTTP authorization, credential use, RP callback or token/nonce/issuer/subject
binding is exercised by these mocks.

Sol4's four null interfaces, preparation/private-input receipt adapter and
partial-ownership cleanup remain separately owned and HELD. This harness
supplies no adapter for them, does not compose them, assumes no MCP receipt
schema and does not touch/contact that worker or report. It starts from declared
synthetic valid/invalid consumer contexts solely to exercise the complete cell.

### Persistent bounds and future receipt protocol

The installed launcher was read/hash-checked as a file, not executed for a
version/probe: `/opt/homebrew/Cellar/node/26.7.0/bin/node`, 50320 bytes,
SHA256 `1ef99ea25fe70c9b67e7efe768ef8ee22148d3cabc703db6131b57aeb617d040`. The already installed Acorn parser uses this Node
installation. The launcher file identity is pinned; its loaded dylib/dependency
closure, actual memory peak and VM/runtime behavior have not been measured or
certified. There is no install, download, alternate Node or automatic fallback.
The future controller rehashes the exact nonsymlink executable before launch.

The controller's earliest clock statement occurs after importing builtin time,
before other controlled preparation. Its START_NS is never reset: assembly,
preflight, Node startup, stdin transport, full-candidate compilation, all cells
and cleanup share START+30 seconds for child work and START+35 seconds for the
outer envelope. The Node performance budget is also persistent across cases/
cells; it is distinct from the synthetic START+840000 / START+900000 clock.
A synchronous VM timeout and async deadline race are supplemental; the
controller's fixed deadline kills/reaps the owned Node group if a VM/async
continuation stalls. No per-cell budget reset or retry can extend the envelope.

The interpreter/tool delivery before that first controller statement and the
final public return transport are not established by a static source check.
The exact timing acceptance sample is after all controller evidence files are
fsynced/closed, before its final finite public return. A blocking filesystem
operation cannot be hard-cancelled by this same controller; lateness therefore
fails the final acceptance check. Owned-child cleanup precedes those writes.
This design does not promise whole60 or a true-first-event browser cleanup.

The future controller requires the exact own worktree and an existing owned
0700 deployment-private directory, plus a fresh root-reserved 16-hex basename.
No evidence is allocated now. It creates one exclusive 0700 subdirectory and
only four exclusive 0600 files: child.stdout, child.stderr, retained.json and
review.json. It never deletes/prunes evidence, changes a config/profile,
touches accepted/main or overwrites an existing path. Free capacity is
UNMEASURED now; at future launch the existing 8 GiB floor is checked, then
sampled every two seconds during child work. The controlled writes are capped
and small; root must review actual capacity when separately releasing runtime.

Exactly one Node child is launched in a new owned process group with stdin/
stdout/stderr pipes, a private empty cwd and only PATH/LANG/LC_ALL in its
environment. No ambient NODE_OPTIONS/NODE_PATH/preload or product secret is
forwarded. Payload delivery and both reads are nonblocking and capped; stdout
65536 bytes, stderr 16384, and each evidence write 262144. A normal EOF does not
substitute for exit: the loop waits for WNOWAIT observation as well as drained
pipes, subject to the fixed deadline. Overflow/incomplete output is retained
as bounded failed evidence, never called a full accepted packet.

Finally keeps the owned leader unreaped for group identity, signals only the
exact group created by this Popen, joins/reaps the child and tests that group
is empty. Unsupported WNOWAIT APIs refuse before launch. An ownership/cleanup
failure retains a fixed failure and prevents success; no discovered PID or
another group is signaled. No daemon/session/browser/app is created or killed.

Actual child exit plus bounded complete raw stdout/stderr and their hashes are
fsynced/closed BEFORE JSON parsing or comparisons. The duplicate-key hook,
closed packet schema, exact source identities/types, fixed case prefixes,
derived group counts, true-int counters, finite elapsed time and finite check
catalog are graded only afterwards. No boolean substitutes for an integer or
source identity. The review journal is explicitly provisional until its close
and final time sample. Final acceptance requires exit 0, full packet, every
case actually completed, no first failure, reaped/empty group, closed evidence
and inclusive within35. There is NO evidence file write/fsync/close, directory
mutation or owned-child operation after final_ns. Only the finite public return
and parent exit follow. Any failure stops this one invocation; root owns
receipt review and any later separately reserved action.

The prospective command carrier is `python3 - <ROOT_RESERVED_FRESH_16HEX>`
with the exact controller below on quoted-heredoc stdin. That single public
basename and the root's separately reviewed immutable source delivery are
required runtime inputs, not requests for a secret/artifact or a permission
implicit in this report. No command was assembled into an executable file or
invoked here. Root must pin this eventual report commit, rehash both archives
and release exactly one run before delivery.

### Complete future installed-Node payload — NOT EXECUTED

The readable logic is the suffix after the BINDING declaration. That declaration
carries canonical base64 candidate/baseline/diff/collector DATA, exact identities,
four inverse spans, fixed case plan and finite check catalog. No omitted function,
hidden helper or mutable source file is needed.

```javascript
import vm from "node:vm";
import {createHash} from "node:crypto";
import {performance} from "node:perf_hooks";
const BINDING = {
  "candidate_b64": "Ly8gUHJvc3BlY3RpdmUgT05FIGZ1bmN0aW9ucy5leGVjIGNlbGw7IE5PVCBleGVjdXRlZCBpbiB0aGlzIGRlc2lnbiBwaGFzZS4KY29uc3QgQ0xPQ0s9ImltcG9ydCBqc29uLHRpbWVcbnByaW50KGpzb24uZHVtcHMoeydtb25vdG9uaWNfbnMnOnN0cih0aW1lLm1vbm90b25pY19ucygpKSwnd2FsbF9lcG9jaF9ucyc6c3RyKHRpbWUudGltZV9ucygpKX0pKVxuIjsKY29uc3QgU1RPUD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN5cyx0aW1lXG5jPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMV0pXG5jbG9jaz17J21vbm90b25pY19ucyc6c3RyKHRpbWUubW9ub3RvbmljX25zKCkpLCd3YWxsX2Vwb2NoX25zJzpzdHIodGltZS50aW1lX25zKCkpfVxucmVjb3JkPXsnc2NoZW1hJzoncmlhdXRoLmQwMS1maXJzdC1vYnNlcnZhdGlvbi92MScsJ2ZpcnN0X2ZhaWx1cmUnOmNbJ2ZpcnN0X2ZhaWx1cmUnXSwnZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyc6Y1snZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyddLCdmaXJzdF9ldmVudF9wcm92ZW4nOkZhbHNlLCdjbG9jayc6Y2xvY2t9XG5ldmVudF9zdGF0ZT0nd3JpdGVfdW5jb25maXJtZWQnXG50cnk6XG4gICAgZmQ9b3Mub3BlbihwYXRobGliLlBhdGgoY1snZXZlbnRfb3V0J10pLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApXG4gICAgd2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgICAgIGYud3JpdGUoanNvbi5kdW1wcyhyZWNvcmQsc29ydF9rZXlzPVRydWUpKydcXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSlcbiAgICBldmVudF9zdGF0ZT0nd3JpdHRlbidcbmV4Y2VwdCBPU0Vycm9yOnBhc3NcbiMgQXR0ZW1wdCBjbG9jayBwZXJzaXN0ZW5jZSBCRUZPUkUgbWFya2VyL3N0YXRlL2J1ZGdldCBjb21wYXJpc29ucy5cbiMgQSBtZXRhZGF0YSBlcnJvciBkb2VzIG5vdCBwcmV2ZW50IHRoZSBvcmlnaW5hbCBlc3NlbnRpYWwgc3RvcCBwcm90b2NvbC5cbmxhYj1wYXRobGliLlBhdGgoY1snbGFiJ10pO21hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbnRyeTpcbiAgICBpZiBsYWIuZXhpc3RzKCk6XG4gICAgICAgIGluZm89bGFiLmxzdGF0KClcbiAgICAgICAgaWYgbm90IHN0YXQuU19JU0RJUihpbmZvLnN0X21vZGUpIG9yIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpIT0wbzcwMCBvciBpbmZvLnN0X3VpZCE9b3MuZ2V0dWlkKCk6XG4gICAgICAgICAgICBtYXJrZXJfc3RhdGU9J293bmVyc2hpcF91bmtub3duJ1xuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgbWFya2VyX3N0YXRlPSdzdG9wX3JlcXVlc3RlZCdcbiAgICAgICAgICAgIGZvciBuYW1lIGluICgoJ3VpLWZhaWx1cmUnLCdzdG9wJykgaWYgY1snZmlyc3RfZmFpbHVyZSddIGlzIG5vdCBOb25lIGVsc2UgKCdzdG9wJywpKTpcbiAgICAgICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgICAgIGZkPW9zLm9wZW4obGFiL25hbWUsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbiAgICAgICAgICAgICAgICAgICAgb3MuY2xvc2UoZmQpXG4gICAgICAgICAgICAgICAgZXhjZXB0IEZpbGVOb3RGb3VuZEVycm9yOm1hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbiAgICAgICAgICAgICAgICBleGNlcHQgRmlsZUV4aXN0c0Vycm9yOnBhc3NcbmV4Y2VwdCBPU0Vycm9yOm1hcmtlcl9zdGF0ZT0nc3RvcF91bmNvbmZpcm1lZCdcbnByaW50KGpzb24uZHVtcHMoeydjbG9jayc6Y2xvY2ssJ2V2ZW50X3N0YXRlJzpldmVudF9zdGF0ZSwnbWFya2VyX3N0YXRlJzptYXJrZXJfc3RhdGV9KSlcbiI7CmNvbnN0IFJFQURCQUNLPSJpbXBvcnQganNvbixvcyxwYXRobGliLHN0YXQsc3VicHJvY2VzcyxzeXMsdGltZVxuYz1qc29uLmxvYWRzKHN5cy5hcmd2WzFdKVxuY2hpbGRyZW49Tm9uZTtoZWxwZXJfcGlkPWNbJ2hlbHBlcl9waWQnXTtvdXRlcl9vYnNlcnZhdGlvbj1Ob25lO3Byb2plY3Rpb249J3VuYXZhaWxhYmxlJ1xuZGVmIGZpbml0ZV9vYnNlcnZhdGlvbih2YWx1ZSk6XG4gICAgaWYgdHlwZSh2YWx1ZSkgaXMgbm90IGRpY3Qgb3Igc2V0KHZhbHVlKSE9IHsnb2JzZXJ2ZWQnLCdkaWFnbm9zdGljJ30gb3IgdHlwZSh2YWx1ZVsnb2JzZXJ2ZWQnXSkgaXMgbm90IGJvb2w6XG4gICAgICAgIHJldHVybiBOb25lXG4gICAgZGlhZ25vc3RpYz12YWx1ZVsnZGlhZ25vc3RpYyddXG4gICAgaWYgZGlhZ25vc3RpYyBpcyBOb25lOlxuICAgICAgICByZXR1cm4geydvYnNlcnZlZCc6dmFsdWVbJ29ic2VydmVkJ10sJ2RpYWdub3N0aWMnOk5vbmV9XG4gICAgaWYgbm90IHZhbHVlWydvYnNlcnZlZCddIG9yIHR5cGUoZGlhZ25vc3RpYykgaXMgbm90IGRpY3Qgb3Igc2V0KGRpYWdub3N0aWMpIT0geydzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnfTpcbiAgICAgICAgcmV0dXJuIE5vbmVcbiAgICBzaXRlcz0oJ2hhbmRsZXInLCdzZXJ2ZXInLCdtYWluJylcbiAgICBraW5kcz0oJ0F0dHJpYnV0ZUVycm9yJywnVHlwZUVycm9yJywnVmFsdWVFcnJvcicsJ0tleUVycm9yJywnT1NFcnJvcicsJ0Jyb2tlblBpcGVFcnJvcicsJ0Nvbm5lY3Rpb25SZXNldEVycm9yJywnVGltZW91dEVycm9yJywnb3RoZXInKVxuICAgIGZ1bmN0aW9ucz0oJ0hlYWRlclJlYWRlci5yZWFkbGluZScsJ0RlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0JywnRGVtby5iZWdpbicsJ0RlbW8uY2FsbGJhY2snLCdEZW1vLmludm9rZScsJ0hhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0JywnSGFuZGxlci5zZW5kX2Vycm9yJywnSGFuZGxlci5nZXQnLCdIYW5kbGVyLnJlcGx5JywnbWFpbicpXG4gICAgc2l0ZSxraW5kLGZ1bmN0aW9uLGxpbmU9KGRpYWdub3N0aWNba10gZm9yIGsgaW4gKCdzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnKSlcbiAgICBpZiB0eXBlKHNpdGUpIGlzIG5vdCBzdHIgb3Igc2l0ZSBub3QgaW4gc2l0ZXMgb3IgdHlwZShraW5kKSBpcyBub3Qgc3RyIG9yIGtpbmQgbm90IGluIGtpbmRzOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIGlmIG5vdCAoKGZ1bmN0aW9uIGlzIE5vbmUgYW5kIGxpbmUgaXMgTm9uZSkgb3IgKHR5cGUoZnVuY3Rpb24pIGlzIHN0ciBhbmQgZnVuY3Rpb24gaW4gZnVuY3Rpb25zIGFuZCB0eXBlKGxpbmUpIGlzIGludCBhbmQgMTw9bGluZTw9MTAyNCkpOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIHJldHVybiB7J29ic2VydmVkJzpUcnVlLCdkaWFnbm9zdGljJzp7J3NpdGUnOnNpdGUsJ2V4Y2VwdGlvbl9jbGFzcyc6a2luZCwnb3duX2Z1bmN0aW9uJzpmdW5jdGlvbiwnb3duX2xpbmUnOmxpbmV9fVxub3V0ZXI9cGF0aGxpYi5QYXRoKGNbJ291dGVyX291dCddKVxuaWYgb3V0ZXIuaXNfZmlsZSgpOlxuICAgIGZkPW9zLm9wZW4ob3V0ZXIsb3MuT19SRE9OTFl8b3MuT19OT0ZPTExPV3xvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbz1vcy5mc3RhdChmZClcbiAgICAgICAgaWYgbm90IChzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKT09MG82MDAgYW5kIDA8aW5mby5zdF9zaXplPD0yNjIxNDQpOlxuICAgICAgICAgICAgcmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIHdpdGggb3MuZmRvcGVuKGZkLCdyYicsY2xvc2VmZD1GYWxzZSkgYXMgc291cmNlOnJhd19ieXRlcz1zb3VyY2UucmVhZCgyNjIxNDUpXG4gICAgICAgIGlmIG5vdCAwPGxlbihyYXdfYnl0ZXMpPD0yNjIxNDQ6cmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIGRhdGE9anNvbi5sb2FkcyhyYXdfYnl0ZXMpXG4gICAgZmluYWxseTpvcy5jbG9zZShmZClcbiAgICBvdXRlcl9vYnNlcnZhdGlvbj1maW5pdGVfb2JzZXJ2YXRpb24oZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbicpKVxuICAgIHByb2plY3Rpb249ZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfb2JzZXJ2YXRpb25fcHJvamVjdGlvbicpXG4gICAgaWYgcHJvamVjdGlvbiBub3QgaW4gKCd1bmF2YWlsYWJsZScsJ3ZhbGlkJywnaW52YWxpZCcpOnByb2plY3Rpb249J2ludmFsaWQnXG4gICAgaWYgcHJvamVjdGlvbj09J3ZhbGlkJyBhbmQgb3V0ZXJfb2JzZXJ2YXRpb24gaXMgTm9uZTpwcm9qZWN0aW9uPSdpbnZhbGlkJ1xuICAgIGFsbG93ZWQ9eyd3aG9hbWknLCdkaXNjb3ZlcnknLCdjb25maWRlbnRpYWxfY2xpZW50X2NyZWF0ZScsJ29wZXJhdG9yX2xvZ2luJywnc2VydmVyJywnbWFpbnRlbmFuY2VfaW5pdCd9XG4gICAgc3RhcnRlZD1kYXRhLmdldCgnaGVscGVyX2ludm9jYXRpb25zJyk9PTEgYW5kIHR5cGUoZGF0YS5nZXQoJ2hlbHBlcl9waWQnKSkgaXMgaW50IGFuZCBkYXRhWydoZWxwZXJfcGlkJ10+MFxuICAgIGlmIHN0YXJ0ZWQ6YWxsb3dlZC5hZGQoJ2hlbHBlcicpXG4gICAgcmF3PWRhdGEuZ2V0KCdvd25lZF9jaGlsZF9leGl0cycpXG4gICAgaWYgaXNpbnN0YW5jZShyYXcsbGlzdCkgYW5kIGxlbihyYXcpPT1sZW4oYWxsb3dlZCkgYW5kIGFsbChpc2luc3RhbmNlKHYsZGljdCkgYW5kIHYuZ2V0KCduYW1lJykgaW4gYWxsb3dlZCBhbmQgdHlwZSh2LmdldCgncGlkJykpIGlzIGludCBhbmQgdlsncGlkJ10+MCBhbmQgdHlwZSh2LmdldCgnZXhpdCcpKSBpcyBpbnQgZm9yIHYgaW4gcmF3KSBhbmQge3ZbJ25hbWUnXSBmb3IgdiBpbiByYXd9PT1hbGxvd2VkIGFuZCBsZW4oe3ZbJ3BpZCddIGZvciB2IGluIHJhd30pPT1sZW4oYWxsb3dlZCkgYW5kIG5leHQodlsncGlkJ10gZm9yIHYgaW4gcmF3IGlmIHZbJ25hbWUnXT09J3NlcnZlcicpPT1jWydzZXJ2ZXJfcGlkJ10gYW5kICgoc3RhcnRlZCBhbmQgbmV4dCh2WydwaWQnXSBmb3IgdiBpbiByYXcgaWYgdlsnbmFtZSddPT0naGVscGVyJyk9PWRhdGFbJ2hlbHBlcl9waWQnXSBhbmQgKGhlbHBlcl9waWQgaXMgTm9uZSBvciBoZWxwZXJfcGlkPT1kYXRhWydoZWxwZXJfcGlkJ10pKSBvciAobm90IHN0YXJ0ZWQgYW5kIGhlbHBlcl9waWQgaXMgTm9uZSBhbmQgZGF0YS5nZXQoJ2hlbHBlcl9pbnZvY2F0aW9ucycpIGlzIE5vbmUgYW5kIGRhdGEuZ2V0KCdoZWxwZXJfcGlkJykgaXMgTm9uZSkpOlxuICAgICAgICBjaGlsZHJlbj1be2s6dltrXSBmb3IgayBpbiAoJ25hbWUnLCdwaWQnLCdleGl0Jyl9IGZvciB2IGluIHJhd11cbiAgICAgICAgaGVscGVyX3BpZD1kYXRhWydoZWxwZXJfcGlkJ10gaWYgc3RhcnRlZCBlbHNlIE5vbmVcbnBpZHM9bGlzdChjWydvd25lZF9waWRzJ10pXG5pZiBoZWxwZXJfcGlkIGlzIG5vdCBOb25lIGFuZCBoZWxwZXJfcGlkIG5vdCBpbiBwaWRzOnBpZHMuYXBwZW5kKGhlbHBlcl9waWQpXG5wPXN1YnByb2Nlc3MucnVuKFsnL2Jpbi9wcycsJy1wJywnLCcuam9pbihzdHIodikgZm9yIHYgaW4gcGlkcyksJy1vJywncGlkPSddLGNhcHR1cmVfb3V0cHV0PVRydWUsdGltZW91dD0zKVxucHNfa25vd249cC5yZXR1cm5jb2RlIGluICgwLDEpIGFuZCBhbGwodi5pc2RpZ2l0KCkgZm9yIHYgaW4gcC5zdGRvdXQuc3BsaXQoKSlcbnByZXNlbnQ9c2V0KGludCh2KSBmb3IgdiBpbiBwLnN0ZG91dC5zcGxpdCgpKSBpZiBwc19rbm93biBlbHNlIHNldCgpXG5wb3J0cz17fVxuZm9yIHBvcnQgaW4gKDkwMDAsMzAwMCk6XG4gICAgcD1zdWJwcm9jZXNzLnJ1bihbJy91c3Ivc2Jpbi9sc29mJywnLW5QJywnLXQnLCctaVRDUDonK3N0cihwb3J0KSwnLXNUQ1A6TElTVEVOJ10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpXG4gICAgcG9ydHNbc3RyKHBvcnQpXT1ub3QgYm9vbChwLnN0ZG91dC5zdHJpcCgpKSBpZiBwLnJldHVybmNvZGUgaW4gKDAsMSkgZWxzZSBOb25lXG5wcmludChqc29uLmR1bXBzKHsnY2xvY2snOnsnbW9ub3RvbmljX25zJzpzdHIodGltZS5tb25vdG9uaWNfbnMoKSksJ3dhbGxfZXBvY2hfbnMnOnN0cih0aW1lLnRpbWVfbnMoKSl9LCdvd25lZF9waWRzJzp7c3RyKHYpOih2IG5vdCBpbiBwcmVzZW50IGlmIHBzX2tub3duIGVsc2UgTm9uZSkgZm9yIHYgaW4gcGlkc30sJ3BvcnRzJzpwb3J0cywnbGFiX2Fic2VudCc6bm90IHBhdGhsaWIuUGF0aChjWydsYWInXSkuZXhpc3RzKCksJ293bmVkX2NoaWxkX2V4aXRzJzpjaGlsZHJlbiwnaGVscGVyX3BpZCc6aGVscGVyX3BpZCwndW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uJzpvdXRlcl9vYnNlcnZhdGlvbiwndW5leHBlY3RlZF9vYnNlcnZhdGlvbl9wcm9qZWN0aW9uJzpwcm9qZWN0aW9ufSkpXG4iOwpjb25zdCBNQVJLRVI9ImltcG9ydCBvcyxwYXRobGliLHN0YXQsc3lzXG5sYWI9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKTtndWFyZD1pbnQoc3lzLmFyZ3ZbMl0pO3NlcnZlcj1pbnQoc3lzLmFyZ3ZbM10pXG5pZiBndWFyZDw9MCBvciBzZXJ2ZXI8PTAgb3IgZ3VhcmQ9PXNlcnZlcjpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKVxuaW5mbz1sYWIubHN0YXQoKVxuaWYgbm90IChzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSk9PTBvNzAwIGFuZCBpbmZvLnN0X3VpZD09b3MuZ2V0dWlkKCkpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5vcy5raWxsKGd1YXJkLDApO29zLmtpbGwoc2VydmVyLDApXG5pZiAobGFiLydzdG9wJykuZXhpc3RzKCkgb3IgKGxhYi8ndWktZmFpbHVyZScpLmV4aXN0cygpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5mZD1vcy5vcGVuKGxhYi8nYnJvd3Nlci1wcmVwYXJlZCcsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbm9zLmNsb3NlKGZkKVxucHJpbnQoJ3tcInByZXBhcmVkX21hcmtlcl93cml0dGVuXCI6dHJ1ZX0nKVxuIjsKY29uc3QgUEVSU0lTVD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzeXNcbnBhdGg9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKVxucmVjb3JkPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMl0pXG4jIENvbnZlcnQgZGVjaW1hbCBzdHJpbmdzIGRpcmVjdGx5IHRvIFB5dGhvbiBpbnRlZ2VycywgYXZvaWRpbmcgSlMgTnVtYmVyIHJvdW5kaW5nLlxuZGVmIGNsb2Nrcyh2YWx1ZSk6XG4gICAgaWYgaXNpbnN0YW5jZSh2YWx1ZSxkaWN0KTpcbiAgICAgICAgZm9yIGssdiBpbiBsaXN0KHZhbHVlLml0ZW1zKCkpOlxuICAgICAgICAgICAgaWYgayBpbiAoJ21vbm90b25pY19ucycsJ3dhbGxfZXBvY2hfbnMnKSBhbmQgaXNpbnN0YW5jZSh2LHN0cik6XG4gICAgICAgICAgICAgICAgYXNzZXJ0IHYuaXNkZWNpbWFsKCkgYW5kIGxlbih2KTw9MTkgYW5kIDA8PWludCh2KTwyKio2M1xuICAgICAgICAgICAgICAgIHZhbHVlW2tdPWludCh2KVxuICAgICAgICAgICAgZWxzZTpjbG9ja3ModilcbiAgICBlbGlmIGlzaW5zdGFuY2UodmFsdWUsbGlzdCk6XG4gICAgICAgIGZvciB2IGluIHZhbHVlOmNsb2Nrcyh2KVxuY2xvY2tzKHJlY29yZClcbmZkPW9zLm9wZW4ocGF0aCxvcy5PX1dST05MWXxvcy5PX0NSRUFUfG9zLk9fRVhDTHxvcy5PX05PRk9MTE9XLDBvNjAwKVxud2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgZi53cml0ZShqc29uLmR1bXBzKHJlY29yZCxzb3J0X2tleXM9VHJ1ZSxpbmRlbnQ9MikrJ1xcbicpO2YuZmx1c2goKTtvcy5mc3luYyhmLmZpbGVubygpKVxucHJpbnQoanNvbi5kdW1wcyh7J3dyaXR0ZW5fZXhjbHVzaXZlJzpUcnVlfSkpXG4iOwpjb25zdCBvd249bG9hZCgiZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIik7CmNvbnN0IE9CU0tFWT0iZDAxX2ltbWVkaWF0ZV9vYnNlcnZhdGlvbl9yZWNvcmQiOwppZiAoIW93biB8fCBvd24ucnVudGltZV9yZWxlYXNlIT09dHJ1ZSB8fCBvd24uZHJpdmVyX293bmVkIT09dHJ1ZSB8fAogICAgb3duLmV4YWN0X2JvdW5kIT09dHJ1ZSB8fCBvd24uc2luZ2xlX2NvbnRyb2xsZXIhPT10cnVlIHx8CiAgICAhWyJwcmVwYXJlZF9lbnRyeSIsImJyb3dzZXJfZGVjaXNpb24iXS5pbmNsdWRlcyhvd24ucGhhc2UpIHx8CiAgICAob3duLnBoYXNlPT09InByZXBhcmVkX2VudHJ5IiYmKG93bi5wcmVwYXJlZF9nYXRlIT09dHJ1ZXx8b3duLmhlbHBlcl9waWQhPT1udWxsfHwKICAgICAgb3duLmZpeHR1cmVfcmVhZHk9PT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mPT09dHJ1ZSkpIHx8CiAgICAob3duLnBoYXNlPT09ImJyb3dzZXJfZGVjaXNpb24iJiYob3duLmZpeHR1cmVfcmVhZHkhPT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mIT09dHJ1ZSkpIHx8CiAgICBvd24uZnJlc2hfcGF0aHNfcHJlZmxpZ2h0IT09dHJ1ZSB8fAogICAgIU51bWJlci5pc1NhZmVJbnRlZ2VyKG93bi5zdGFydF9lcG9jaF9tcykgfHwgdHlwZW9mIG93bi53b3Jrc3BhY2UhPT0ic3RyaW5nIiB8fAogICAgbmV3IFNldChbb3duLmd1YXJkX3BpZCxvd24uc2VydmVyX3BpZCxvd24uYnJvd3Nlcl9waWQsCiAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSkuc2l6ZSE9PShvd24uaGVscGVyX3BpZD09PW51bGw/Mzo0KSB8fAogICAgIVtvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCxvd24uZXhlY19zZXNzaW9uLAogICAgICAgLi4uKG93bi5oZWxwZXJfcGlkPT09bnVsbD9bXTpbb3duLmhlbHBlcl9waWRdKV0KICAgICAgIC5ldmVyeSh2PT5OdW1iZXIuaXNTYWZlSW50ZWdlcih2KSYmdj4wKSB8fAogICAgIVtvd24uc2Vzc2lvbixvd24udGFyZ2V0X2lkLG93bi50YWJfaWQsb3duLmxhYixvd24ub3V0ZXJfb3V0LAogICAgICAgb3duLmV2ZW50X291dCxvd24uY2xlYW51cF9vdXRdLmV2ZXJ5KHY9PnR5cGVvZiB2PT09InN0cmluZyImJnYubGVuZ3RoPjApKSB7CiAgdGV4dCh7cHJvcG9zYWxfcmVmdXNlZDoiZnJlc2hfb3duZWRfY29udGV4dF9yZXF1aXJlZCJ9KTtleGl0KCk7Cn0KY29uc3QgcHJpb3I9bG9hZChPQlNLRVkpOwppZihvd24uY2xvc2VkPT09dHJ1ZXx8cHJpb3I/LmNsZWFudXBfc3RhcnRlZD09PXRydWUpewogIHRleHQoe3Byb3Bvc2FsX3JlZnVzZWQ6ImZpeHR1cmVfYWxyZWFkeV9zdG9wcGVkIixyZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpmYWxzZX0pO2V4aXQoKTsKfQpjb25zdCByZWNvcmQ9cHJpb3I/P3sKICBzY2hlbWE6InJpYXV0aC5kMDEtaW1tZWRpYXRlLW9ic2VydmF0aW9uLWNsZWFudXAvdjEiLAogIGZpcnN0X2ZhaWx1cmU6bnVsbCxmaXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOm51bGwsCiAgZmlyc3RfZXZlbnRfcHJvdmVuOmZhbHNlLHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogIHVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbjpudWxsLGRpYWdub3N0aWNfZXJyb3JzOltdLGNsZWFudXBfc3RhcnRlZDpmYWxzZSwKICBmaXJzdF9jbG9jazpudWxsLG9ic2VydmF0aW9uX3JlY2VpcHRzOltdLGNvbnRyb2xsZXJfb2JzZXJ2YXRpb25zOltdLGxvY2FsX3Rvb2xfcmVjZWlwdHM6W10sYWN0aW9uczpbXSxjbGVhbnVwX2Vycm9yczpbXSwKICBmaW5hbF9hYnNlbmNlOm51bGwsb3duZWRfY2hpbGRfZXhpdHM6bnVsbCxjb250cm9sbGVyX2V4aXQ6bnVsbCwKICBmaXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zOm51bGwsbGF0Y2hfdG9fYWJzZW5jZV9tczpudWxsCn07CmNvbnN0IHNxPXM9PiInIitTdHJpbmcocykucmVwbGFjZSgvJy9nLCInXFwnJyIpKyInIjsKY29uc3QgcmV0YWluPSgpPT5zdG9yZShPQlNLRVkscmVjb3JkKTsKY29uc3QgY2xlYW51cEVycm9yPWxhYmVsPT57CiAgaWYoIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5pbmNsdWRlcyhsYWJlbCkpcmVjb3JkLmNsZWFudXBfZXJyb3JzLnB1c2gobGFiZWwpOwogIHJldGFpbigpOwp9Owpjb25zdCBsb2NhbD1hc3luYyhzb3VyY2UsYXJnKT0+ewogIGNvbnN0IGNtZD0icHl0aG9uMyAtYyAiK3NxKHNvdXJjZSkrKGFyZz09PXVuZGVmaW5lZD8iIjoiICIrc3EoSlNPTi5zdHJpbmdpZnkoYXJnKSkpOwogIGNvbnN0IHI9YXdhaXQgdG9vbHMuZXhlY19jb21tYW5kKHtjbWQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgeWllbGRfdGltZV9tczoxMDAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7CiAgICBsYWJlbDpzb3VyY2U9PT1DTE9DSz8iY2xvY2siOnNvdXJjZT09PVNUT1A/InN0b3AiOnNvdXJjZT09PVJFQURCQUNLPyJyZWFkYmFjayI6InVua25vd24iLAogICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGwKICB9KTtyZXRhaW4oKTsgLy8gTnVtZXJpYyBjb2xsZWN0b3IgcmVjZWlwdCBCRUZPUkUgY29tcGFyaXNvbnMuCiAgaWYoci5zZXNzaW9uX2lkIT09dW5kZWZpbmVkKSB7CiAgICBjbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTt0aHJvdyBuZXcgRXJyb3IoImxvY2FsX3BlbmRpbmciKTsKICB9CiAgaWYoci5leGl0X2NvZGUhPT0wKXRocm93IG5ldyBFcnJvcigibG9jYWxfZmFpbGVkIik7CiAgcmV0dXJuIEpTT04ucGFyc2Uoci5vdXRwdXQpOwp9OwpmdW5jdGlvbiBmaW5pdGVPYnNlcnZhdGlvbih2YWx1ZSkgewogIGNvbnN0IGV4YWN0PSh2LGtleXMpPT52IT09bnVsbCYmdHlwZW9mIHY9PT0ib2JqZWN0IiYmIUFycmF5LmlzQXJyYXkodikmJgogICAgT2JqZWN0LmtleXModikubGVuZ3RoPT09a2V5cy5sZW5ndGgmJmtleXMuZXZlcnkoaz0+T2JqZWN0Lmhhc093bih2LGspKTsKICBpZighZXhhY3QodmFsdWUsWyJvYnNlcnZlZCIsImRpYWdub3N0aWMiXSl8fHR5cGVvZiB2YWx1ZS5vYnNlcnZlZCE9PSJib29sZWFuIilyZXR1cm4gbnVsbDsKICBjb25zdCBkPXZhbHVlLmRpYWdub3N0aWM7CiAgaWYoZD09PW51bGwpcmV0dXJuIHtvYnNlcnZlZDp2YWx1ZS5vYnNlcnZlZCxkaWFnbm9zdGljOm51bGx9OwogIGlmKCF2YWx1ZS5vYnNlcnZlZHx8IWV4YWN0KGQsWyJzaXRlIiwiZXhjZXB0aW9uX2NsYXNzIiwib3duX2Z1bmN0aW9uIiwib3duX2xpbmUiXSl8fAogICAgICFbImhhbmRsZXIiLCJzZXJ2ZXIiLCJtYWluIl0uaW5jbHVkZXMoZC5zaXRlKXx8CiAgICAgIVsiQXR0cmlidXRlRXJyb3IiLCJUeXBlRXJyb3IiLCJWYWx1ZUVycm9yIiwiS2V5RXJyb3IiLCJPU0Vycm9yIiwiQnJva2VuUGlwZUVycm9yIiwKICAgICAgICJDb25uZWN0aW9uUmVzZXRFcnJvciIsIlRpbWVvdXRFcnJvciIsIm90aGVyIl0uaW5jbHVkZXMoZC5leGNlcHRpb25fY2xhc3MpKXJldHVybiBudWxsOwogIGNvbnN0IGZ1bmN0aW9ucz1bIkhlYWRlclJlYWRlci5yZWFkbGluZSIsIkRlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0IiwiRGVtby5iZWdpbiIsIkRlbW8uY2FsbGJhY2siLAogICAgIkRlbW8uaW52b2tlIiwiSGFuZGxlci5oYW5kbGVfb25lX3JlcXVlc3QiLCJIYW5kbGVyLnNlbmRfZXJyb3IiLCJIYW5kbGVyLmdldCIsIkhhbmRsZXIucmVwbHkiLCJtYWluIl07CiAgaWYoISgoZC5vd25fZnVuY3Rpb249PT1udWxsJiZkLm93bl9saW5lPT09bnVsbCl8fAogICAgICAgKGZ1bmN0aW9ucy5pbmNsdWRlcyhkLm93bl9mdW5jdGlvbikmJk51bWJlci5pc1NhZmVJbnRlZ2VyKGQub3duX2xpbmUpJiYKICAgICAgICBkLm93bl9saW5lPj0xJiZkLm93bl9saW5lPD0xMDI0KSkpcmV0dXJuIG51bGw7CiAgcmV0dXJuIHtvYnNlcnZlZDp0cnVlLGRpYWdub3N0aWM6e3NpdGU6ZC5zaXRlLGV4Y2VwdGlvbl9jbGFzczpkLmV4Y2VwdGlvbl9jbGFzcywKICAgIG93bl9mdW5jdGlvbjpkLm93bl9mdW5jdGlvbixvd25fbGluZTpkLm93bl9saW5lfX07Cn0KZnVuY3Rpb24gZGlhZ25vc3RpYyh2YWx1ZSxwcm9qZWN0aW9uKSB7CiAgY29uc3QgcHJvamVjdGVkPWZpbml0ZU9ic2VydmF0aW9uKHZhbHVlKTsKICBpZihwcm9qZWN0aW9uPT09ImludmFsaWQifHwhWyJ2YWxpZCIsInVuYXZhaWxhYmxlIl0uaW5jbHVkZXMocHJvamVjdGlvbil8fAogICAgIChwcm9qZWN0aW9uPT09InZhbGlkIiYmcHJvamVjdGVkPT09bnVsbCkpIHsKICAgIGlmKCFyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMuaW5jbHVkZXMoIm9ic2VydmVyX3Byb2plY3Rpb25faW52YWxpZCIpKQogICAgICByZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMucHVzaCgib2JzZXJ2ZXJfcHJvamVjdGlvbl9pbnZhbGlkIik7CiAgfQogIGlmKHByb2plY3RlZCE9PW51bGwmJnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb249PT1udWxsKQogICAgcmVjb3JkLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbj1wcm9qZWN0ZWQ7CiAgcmV0YWluKCk7IC8vIE5vIHJhdyBkaWFnbm9zdGljIGZhbGxiYWNrIGFuZCBubyBmaXJzdC1mYWlsdXJlIG9yIG91dGNvbWUgbXV0YXRpb24uCn0KY29uc3QgbnM9Yz0+QmlnSW50KGMubW9ub3RvbmljX25zKTsKY29uc3QgZ2V0Q2xvY2s9KCk9PmxvY2FsKENMT0NLKTsKbGV0IGNvbnRyb2xsZXJKb2luZWQ9ZmFsc2U7CmxldCBjb250cm9sbGVyUG9sbEF2YWlsYWJsZT10cnVlOwpsZXQgY29udHJvbGxlckJ1ZmZlcj0iIjsKbGV0IGNsZWFudXBTdGFydGVkPWZhbHNlOwpsZXQgbGFzdFNuYXBzaG90RmxhZ3M9bnVsbDsKZnVuY3Rpb24gbGF0Y2gobGFiZWwscmVjZWl2ZWRXYWxsKSB7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICByZWNvcmQuZmlyc3RfZmFpbHVyZT1sYWJlbDsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPXJlY2VpdmVkV2FsbDsKICAgIHJldGFpbigpOyAvLyBTeW5jaHJvbm91cyBmaXJzdC1mYWlsdXJlL3dhbGwgbGF0Y2ggQkVGT1JFIGFueSBuZXcgYXdhaXQvb3V0cHV0LgogIH0KfQpmdW5jdGlvbiBvYnNlcnZlQ29udHJvbGxlcihyLHJlY2VpdmVkV2FsbCkgewogIGNvbnN0IG9ic2VydmF0aW9uPXtyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbCwKICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgIGhlbHBlcl9jb21wbGV0ZWQ6ZmFsc2UsaGVscGVyX2V4aXQ6bnVsbCxmaXh0dXJlX2ZpbmlzaGVkOmZhbHNlfTsKICAvLyBDb21wbGV0ZSBudW1lcmljIHJlc3VsdCBwcm9qZWN0aW9uIGlzIHJldGFpbmVkIEJFRk9SRSBjb21wYXJpc29ucy4KICByZWNvcmQuY29udHJvbGxlcl9vYnNlcnZhdGlvbnMucHVzaChvYnNlcnZhdGlvbik7cmV0YWluKCk7CiAgaWYodHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciIpIHsKICAgIGNvbnRyb2xsZXJKb2luZWQ9dHJ1ZTtyZWNvcmQuY29udHJvbGxlcl9leGl0PXIuZXhpdF9jb2RlO3JldGFpbigpOwogIH0KICBjb250cm9sbGVyQnVmZmVyKz10eXBlb2Ygci5vdXRwdXQ9PT0ic3RyaW5nIj9yLm91dHB1dDoiIjsKICBpZihjb250cm9sbGVyQnVmZmVyLmxlbmd0aD4xNjM4NCkgewogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250cm9sbGVyQnVmZmVyPSIiO3JldHVybjsKICB9CiAgbGV0IGN1dDsKICB3aGlsZSgoY3V0PWNvbnRyb2xsZXJCdWZmZXIuaW5kZXhPZigiXG4iKSk+PTApIHsKICAgIGNvbnN0IGxpbmU9Y29udHJvbGxlckJ1ZmZlci5zbGljZSgwLGN1dCk7Y29udHJvbGxlckJ1ZmZlcj1jb250cm9sbGVyQnVmZmVyLnNsaWNlKGN1dCsxKTsKICAgIGlmKCFsaW5lLnRyaW0oKSljb250aW51ZTsKICAgIGxldCBldmVudDsKICAgIHRyeXtldmVudD1KU09OLnBhcnNlKGxpbmUpO31jYXRjaHsKICAgICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250aW51ZTsKICAgIH0KICAgIGlmKE9iamVjdC5oYXNPd24oZXZlbnQsInVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiIpKQogICAgICBkaWFnbm9zdGljKGV2ZW50LnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbixldmVudC51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoZXZlbnQuZml4dHVyZV9yZWFkeT09PXRydWUpIHsKICAgICAgaWYoZXZlbnQuZ3VhcmRfcGlkIT09b3duLmd1YXJkX3BpZHx8ZXZlbnQuc2VydmVyX3BpZCE9PW93bi5zZXJ2ZXJfcGlkfHwKICAgICAgICAgZXZlbnQubGFiIT09b3duLmxhYnx8IU51bWJlci5pc1NhZmVJbnRlZ2VyKGV2ZW50LmhlbHBlcl9waWQpfHxldmVudC5oZWxwZXJfcGlkPD0wfHwKICAgICAgICAgW293bi5ndWFyZF9waWQsb3duLnNlcnZlcl9waWQsb3duLmJyb3dzZXJfcGlkXS5pbmNsdWRlcyhldmVudC5oZWxwZXJfcGlkKXx8CiAgICAgICAgIChvd24uaGVscGVyX3BpZCE9PW51bGwmJm93bi5oZWxwZXJfcGlkIT09ZXZlbnQuaGVscGVyX3BpZCkpCiAgICAgICAgbGF0Y2goImZpeHR1cmVfcmVhZHlfdW5jb25maXJtZWQiLHJlY2VpdmVkV2FsbCk7CiAgICAgIGVsc2UgewogICAgICAgIG93bi5oZWxwZXJfcGlkPWV2ZW50LmhlbHBlcl9waWQ7b3duLmZpeHR1cmVfcmVhZHk9dHJ1ZTtvd24ubGlzdGVuZXJfcGlkX3Byb29mPXRydWU7CiAgICAgICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICAgICAgfQogICAgfQogICAgaWYoZXZlbnQuaGVscGVyX2NvbXBsZXRlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uaGVscGVyX2NvbXBsZXRlZD10cnVlOwogICAgICBvYnNlcnZhdGlvbi5oZWxwZXJfZXhpdD10eXBlb2YgZXZlbnQuZXhpdD09PSJudW1iZXIiP2V2ZW50LmV4aXQ6bnVsbDsKICAgICAgcmV0YWluKCk7CiAgICAgIGlmKGV2ZW50LmV4aXQhPT0wKWxhdGNoKCJoZWxwZXJfZmFpbGVkIixyZWNlaXZlZFdhbGwpOwogICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSYmCiAgICAgICAgICAgICAgIShvd24ucGhhc2U9PT0iYnJvd3Nlcl9kZWNpc2lvbiImJgogICAgICAgICAgICAgICAgb3duLm5leHRfZGVjaXNpb24/LmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIikpCiAgICAgICAgbGF0Y2goImhlbHBlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50IixyZWNlaXZlZFdhbGwpOwogICAgfQogICAgaWYoZXZlbnQuZml4dHVyZV9maW5pc2hlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uZml4dHVyZV9maW5pc2hlZD10cnVlO3JldGFpbigpOwogICAgICBpZighY2xlYW51cFN0YXJ0ZWQpbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKICAgIH0KICB9CiAgaWYoY29udHJvbGxlckpvaW5lZCYmIWNsZWFudXBTdGFydGVkKQogICAgbGF0Y2goImNvbnRyb2xsZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIscmVjZWl2ZWRXYWxsKTsKfQphc3luYyBmdW5jdGlvbiBwb2xsQ29udHJvbGxlcigpIHsKICBpZihjb250cm9sbGVySm9pbmVkfHwhY29udHJvbGxlclBvbGxBdmFpbGFibGUpcmV0dXJuOwogIHRyeSB7CiAgICBjb25zdCByPWF3YWl0IHRvb2xzLndyaXRlX3N0ZGluKHtzZXNzaW9uX2lkOm93bi5leGVjX3Nlc3Npb24sCiAgICAgIGNoYXJzOiIiLHlpZWxkX3RpbWVfbXM6NTAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgICBjb25zdCByZWNlaXZlZFdhbGw9RGF0ZS5ub3coKTsKICAgIG9ic2VydmVDb250cm9sbGVyKHIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHsKICAgIGNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlPWZhbHNlOwogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25fdW5hdmFpbGFibGUiLERhdGUubm93KCkpOwogICAgaWYoY2xlYW51cFN0YXJ0ZWQpY2xlYW51cEVycm9yKCJjb250cm9sbGVyX2pvaW5fdW5jb25maXJtZWQiKTsKICB9Cn0KYXN5bmMgZnVuY3Rpb24gdGltZWREcml2ZXIobGFiZWwsYWxsb2NhdGlvbixvcGVyYXRpb24scHJvamVjdCkgewogIGxldCBzdGFydD1udWxsLGVuZD1udWxsLHJlc3VsdD1udWxsLHN0YXRlPSJ1bmtub3duIjsKICB0cnl7c3RhcnQ9YXdhaXQgZ2V0Q2xvY2soKTt9Y2F0Y2h7Y2xlYW51cEVycm9yKCJjbG9ja191bmF2YWlsYWJsZSIpO30KICBjb25zdCBhY3Rpb249e2xhYmVsLHN0YXJ0X2Nsb2NrOnN0YXJ0LGVuZF9jbG9jazpudWxsLGFsbG93ZWRfbXM6YWxsb2NhdGlvbiwKICAgIHJlc3VsdF9zdGF0ZToidW5rbm93biIsZWxhcHNlZF9tczpudWxsLG92ZXJfYnVkZ2V0Om51bGx9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsgLy8gU3RhcnQgcmV0YWluZWQgQkVGT1JFIGVudGVyaW5nIERyaXZlciBjYWxsLgogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7CiAgICBzdGF0ZT1yZXN1bHQ/LmlzRXJyb3I9PT10cnVlPyJyZWZ1c2VkIjoicmV0dXJuZWQiOwogIH0gY2F0Y2gge3N0YXRlPSJleGNlcHRpb24iO30KICB0cnl7ZW5kPWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgYWN0aW9uLmVuZF9jbG9jaz1lbmQ7YWN0aW9uLnJlc3VsdF9zdGF0ZT1zdGF0ZTsKICByZXRhaW4oKTsgLy8gRW5kL3Jlc3VsdCByZXRhaW5lZCBCRUZPUkUgYnVkZ2V0IGNvbXBhcmlzb24uCiAgaWYoc3RhcnQmJmVuZCkgewogICAgY29uc3QgZD1ucyhlbmQpLW5zKHN0YXJ0KTsKICAgIGlmKGQ+PTBuKSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcihkLzEwMDAwMDBuKTsKICAgICAgYWN0aW9uLm92ZXJfYnVkZ2V0PWFjdGlvbi5lbGFwc2VkX21zPmFsbG9jYXRpb247CiAgICB9IGVsc2UgY2xlYW51cEVycm9yKCJjbG9ja19pbnZhbGlkIik7CiAgfQogIGlmKGFjdGlvbi5vdmVyX2J1ZGdldCljbGVhbnVwRXJyb3IoImRyaXZlcl9vcGVyYXRpb25fb3Zlcl9idWRnZXQiKTsKICBpZihzdGF0ZSE9PSJyZXR1cm5lZCIpY2xlYW51cEVycm9yKCJkcml2ZXJfb3BlcmF0aW9uX3VuY29uZmlybWVkIik7CiAgaWYocHJvamVjdClwcm9qZWN0KHJlc3VsdCxzdGF0ZSk7CiAgcmV0YWluKCk7Cn0KYXN5bmMgZnVuY3Rpb24gY2xlYW51cCgpIHsKICBzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsgLy8gQ2xlYXIgYmVmb3JlIGFueSBjbGVhbnVwIGF3YWl0LgogIGlmKGNsZWFudXBTdGFydGVkKXJldHVybjsKICBjbGVhbnVwU3RhcnRlZD10cnVlO3JlY29yZC5jbGVhbnVwX3N0YXJ0ZWQ9dHJ1ZTtyZXRhaW4oKTsKICB0cnkgewogICAgY29uc3Qgcj1hd2FpdCBsb2NhbChTVE9QLHsKICAgICAgbGFiOm93bi5sYWIsZXZlbnRfb3V0Om93bi5ldmVudF9vdXQsCiAgICAgIGZpcnN0X2ZhaWx1cmU6cmVjb3JkLmZpcnN0X2ZhaWx1cmUsCiAgICAgIGZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXM6cmVjb3JkLmZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXMKICAgIH0pOwogICAgcmVjb3JkLmZpcnN0X2Nsb2NrPXIuY2xvY2s7cmVjb3JkLnN0b3Bfc3RhdGU9ci5tYXJrZXJfc3RhdGU7cmV0YWluKCk7CiAgICBpZihyLmV2ZW50X3N0YXRlIT09IndyaXR0ZW4iKWNsZWFudXBFcnJvcigib2JzZXJ2YXRpb25fbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICAgIGlmKHIubWFya2VyX3N0YXRlPT09InN0b3BfdW5jb25maXJtZWQiKWNsZWFudXBFcnJvcigic3RvcF91bmNvbmZpcm1lZCIpOwogICAgaWYoci5tYXJrZXJfc3RhdGU9PT0ib3duZXJzaGlwX3Vua25vd24iKWNsZWFudXBFcnJvcigib3duZXJzaGlwX3Vua25vd24iKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoInN0b3Bfb3JfY2xvY2tfcmVjb3JkX3VuYXZhaWxhYmxlIik7fQogIC8vIE5vIG91dHN0YW5kaW5nIERyaXZlciBjYWxsIGV4aXN0cyBoZXJlOiBhbGwgcGFnZSBjYWxscyBhYm92ZSB3ZXJlIGF3YWl0ZWQuCiAgLy8gT3JpZ2luYWwgc3RvcCBwcm90b2NvbCBhbHJlYWR5IGNhdXNlcyB0aGUgY29udHJvbGxlcidzIG93bmVkLWNoaWxkIGZpbmFsbHkuCiAgYXdhaXQgdGltZWREcml2ZXIoImtpbGxfYXBwIiwzMDAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2tpbGxfYXBwKHtwaWQ6b3duLmJyb3dzZXJfcGlkfSkpOwogIGF3YWl0IHRpbWVkRHJpdmVyKCJlbmRfc2Vzc2lvbiIsMTUwMDAsCiAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19lbmRfc2Vzc2lvbih7c2Vzc2lvbjpvd24uc2Vzc2lvbn0pLChyLHN0YXRlKT0+ewogICAgICByZWNvcmQuc2Vzc2lvbl9lbmRlZD1zdGF0ZT09PSJyZXR1cm5lZCImJgogICAgICAgIHI/LnN0cnVjdHVyZWRDb250ZW50Py5hY3RpdmU9PT1mYWxzZSYmci5zdHJ1Y3R1cmVkQ29udGVudC5zZXNzaW9uPT09b3duLnNlc3Npb247CiAgICAgIHJldGFpbigpOwogICAgfSk7CiAgYXdhaXQgdGltZWREcml2ZXIoImxpc3Rfd2luZG93cyIsNTAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2xpc3Rfd2luZG93cyh7cGlkOm93bi5icm93c2VyX3BpZH0pLChyLHN0YXRlKT0+ewogICAgICBjb25zdCB3aW5kb3dzPXI/LnN0cnVjdHVyZWRDb250ZW50Py53aW5kb3dzOwogICAgICByZWNvcmQud2luZG93X2NvdW50PXN0YXRlPT09InJldHVybmVkIiYmQXJyYXkuaXNBcnJheSh3aW5kb3dzKT93aW5kb3dzLmxlbmd0aDpudWxsOwogICAgICByZXRhaW4oKTsKICAgIH0pOwogIGxldCByZWFkU3RhcnQ9bnVsbDsKICB0cnl7cmVhZFN0YXJ0PWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgY29uc3QgYWN0aW9uPXtsYWJlbDoib3duZWRfam9pbl9yZWFkYmFjayIsc3RhcnRfY2xvY2s6cmVhZFN0YXJ0LGVuZF9jbG9jazpudWxsLAogICAgYWxsb3dlZF9tczpudWxsLGVsYXBzZWRfbXM6bnVsbCxvdmVyX2J1ZGdldDpudWxsLHJlc3VsdF9zdGF0ZToidW5rbm93biJ9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsKICBpZihyZWFkU3RhcnQmJnJlY29yZC5maXJzdF9jbG9jaykgewogICAgYWN0aW9uLmFsbG93ZWRfbXM9TWF0aC5tYXgoMCw2MDAwMC1OdW1iZXIoKG5zKHJlYWRTdGFydCktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pKTsKICAgIHJldGFpbigpOwogIH0KICAvLyBDb250aW51ZSBlc3NlbnRpYWwgam9pbiBhZnRlcjYwcyBpZiBsYXRlOyBuZXZlciBjbGFpbSB0aGF0IGRlYWRsaW5lIGVuZm9yY2VkLgogIC8vIERvIG5vdCBwb2xsIGFub3RoZXIgZXhlYyBzZXNzaW9uIG9yIHNlbmQgYSBtYW51YWwgcHJvY2VzcyBzaWduYWwuCiAgd2hpbGUoIWNvbnRyb2xsZXJKb2luZWQmJmNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlJiZEYXRlLm5vdygpPG93bi5zdGFydF9lcG9jaF9tcys5MDAwMDApIHsKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spIHsKICAgICAgdHJ5IHsKICAgICAgICBjb25zdCBjPWF3YWl0IGdldENsb2NrKCk7cmVjb3JkLmxhc3Rfam9pbl9jbG9jaz1jO3JldGFpbigpOwogICAgICAgIGlmKG5zKGMpLW5zKHJlY29yZC5maXJzdF9jbG9jayk+NjAwMDAwMDAwMDBuKQogICAgICAgICAgY2xlYW51cEVycm9yKCJjbGVhbnVwX2J1ZGdldF9leGNlZWRlZCIpOwogICAgICB9IGNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgICB9CiAgfQogIGlmKCFjb250cm9sbGVySm9pbmVkKWNsZWFudXBFcnJvcigiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIik7CiAgdHJ5IHsKICAgIGNvbnN0IHI9YXdhaXQgbG9jYWwoUkVBREJBQ0ssewogICAgICBvd25lZF9waWRzOltvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCwKICAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSwKICAgICAgbGFiOm93bi5sYWIsb3V0ZXJfb3V0Om93bi5vdXRlcl9vdXQsaGVscGVyX3BpZDpvd24uaGVscGVyX3BpZCxzZXJ2ZXJfcGlkOm93bi5zZXJ2ZXJfcGlkCiAgICB9KTsKICAgIGFjdGlvbi5lbmRfY2xvY2s9ci5jbG9jazthY3Rpb24ucmVzdWx0X3N0YXRlPSJyZXR1cm5lZCI7CiAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHM9ci5vd25lZF9jaGlsZF9leGl0czsKICAgIGRpYWdub3N0aWMoci51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sci51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoTnVtYmVyLmlzU2FmZUludGVnZXIoci5oZWxwZXJfcGlkKSYmci5oZWxwZXJfcGlkPjApb3duLmhlbHBlcl9waWQ9ci5oZWxwZXJfcGlkOwogICAgcmVjb3JkLmZpbmFsX2Fic2VuY2U9e293bmVkX3BpZHM6ci5vd25lZF9waWRzLHBvcnRzOnIucG9ydHMsCiAgICAgIGxhYjpyLmxhYl9hYnNlbnQsd2luZG93X2NvdW50OnJlY29yZC53aW5kb3dfY291bnQ/P251bGwsCiAgICAgIHNlc3Npb25fZW5kZWQ6cmVjb3JkLnNlc3Npb25fZW5kZWQ9PT10cnVlfTsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zPXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPT09bnVsbD9udWxsOgogICAgICBEYXRlLm5vdygpLXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOwogICAgcmV0YWluKCk7IC8vIEZ1bGwgZml4ZWQgcmVhZGJhY2svZXhpdHMvY2xvY2sgQkVGT1JFIGNvbXBhcmlzb25zLgogICAgaWYocmVhZFN0YXJ0KSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVhZFN0YXJ0KSkvMTAwMDAwMG4pOwogICAgICBhY3Rpb24ub3Zlcl9idWRnZXQ9YWN0aW9uLmFsbG93ZWRfbXM9PT1udWxsP251bGw6YWN0aW9uLmVsYXBzZWRfbXM+YWN0aW9uLmFsbG93ZWRfbXM7CiAgICB9CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spCiAgICAgIHJlY29yZC5sYXRjaF90b19hYnNlbmNlX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pOwogICAgaWYoYWN0aW9uLm92ZXJfYnVkZ2V0KWNsZWFudXBFcnJvcigiY2xlYW51cF9idWRnZXRfZXhjZWVkZWQiKTsKICAgIGNvbnN0IGE9cmVjb3JkLmZpbmFsX2Fic2VuY2U7CiAgICBpZighY29udHJvbGxlckpvaW5lZHx8IUFycmF5LmlzQXJyYXkocmVjb3JkLm93bmVkX2NoaWxkX2V4aXRzKXx8CiAgICAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHMubGVuZ3RoIT09KG93bi5oZWxwZXJfcGlkPT09bnVsbD82OjcpKQogICAgICBjbGVhbnVwRXJyb3IoImNoaWxkX3JlYXBfaW5jb21wbGV0ZSIpOwogICAgaWYoIU9iamVjdC52YWx1ZXMoYS5vd25lZF9waWRzKS5ldmVyeSh2PT52PT09dHJ1ZSl8fAogICAgICAgIU9iamVjdC52YWx1ZXMoYS5wb3J0cykuZXZlcnkodj0+dj09PXRydWUpfHxhLmxhYiE9PXRydWV8fAogICAgICAgYS53aW5kb3dfY291bnQhPT0wfHxhLnNlc3Npb25fZW5kZWQhPT10cnVlKQogICAgICBjbGVhbnVwRXJyb3IoIm93bmVkX3Jlc291cmNlX2Fic2VuY2VfdW5wcm92ZW4iKTsKICB9IGNhdGNoIHthY3Rpb24ucmVzdWx0X3N0YXRlPSJleGNlcHRpb24iO2NsZWFudXBFcnJvcigib3duZWRfcmVhZGJhY2tfdW5hdmFpbGFibGUiKTt9CiAgY2xlYW51cEVycm9yKCJmaXJzdF9ldmVudF91bnByb3ZlbiIpOwogIHJlY29yZC53aG9sZV9jbGVhbnVwX3dpdGhpbjYwX3Byb3Zlbj1mYWxzZTtyZXRhaW4oKTsKICAvLyBFeGNsdXNpdmUgbmV3IGZpbGUgb25seTsgZXhpc3Rpbmcgb3V0ZXIvaGVscGVyL3Byb3ZpZGVyIGV2aWRlbmNlIHVudG91Y2hlZC4KICB0cnkgewogICAgY29uc3QgY21kPSJweXRob24zIC1jICIrc3EoUEVSU0lTVCkrIiAiK3NxKG93bi5jbGVhbnVwX291dCkrIiAiK3NxKEpTT04uc3RyaW5naWZ5KHJlY29yZCkpOwogICAgY29uc3Qgcj1hd2FpdCB0b29scy5leGVjX2NvbW1hbmQoe2NtZCx3b3JrZGlyOm93bi53b3Jrc3BhY2UsCiAgICAgIHlpZWxkX3RpbWVfbXM6MTAwMDAsbWF4X291dHB1dF90b2tlbnM6NTAwfSk7CiAgICByZWNvcmQubG9jYWxfdG9vbF9yZWNlaXB0cy5wdXNoKHtsYWJlbDoicGVyc2lzdCIsCiAgICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGx9KTtyZXRhaW4oKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZHx8ci5leGl0X2NvZGUhPT0wKQogICAgICBjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTt9CiAgY29udHJvbGxlckJ1ZmZlcj0iIjtvd24uY2xvc2VkPXRydWU7c3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKfQphc3luYyBmdW5jdGlvbiBjaGVja2VkKGxhYmVsLG9wZXJhdGlvbixwcmVkaWNhdGUpIHsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpcmV0dXJuIG51bGw7CiAgaWYoRGF0ZS5ub3coKT49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkgewogICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuIG51bGw7CiAgfQogIGxldCByZXN1bHQscmVjZWl2ZWRXYWxsOwogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7cmVjZWl2ZWRXYWxsPURhdGUubm93KCk7CiAgICByZWNvcmQub2JzZXJ2YXRpb25fcmVjZWlwdHMucHVzaCh7a2luZDpsYWJlbCxyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbH0pO3JldGFpbigpOwogICAgaWYocmVzdWx0Py5pc0Vycm9yPT09dHJ1ZSlsYXRjaCgiYnJvd3Nlcl90b29sX3JlZnVzZWQiLHJlY2VpdmVkV2FsbCk7CiAgICBlbHNlIGlmKCFwcmVkaWNhdGUocmVzdWx0KSlsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHtsYXRjaCgiYnJvd3Nlcl90b29sX29yX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7fQogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVhbnVwKCk7CiAgcmV0dXJuIHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbD9yZXN1bHQ6bnVsbDsKfQpjb25zdCBzbmFwc2hvdEFyZ3M9e3Nlc3Npb246b3duLnNlc3Npb24sdGFyZ2V0X2lkOm93bi50YXJnZXRfaWQsdGFiX2lkOm93bi50YWJfaWQsCiAgc25hcHNob3RfZm9ybWF0OiJzZW1hbnRpY192MiIsaW5jbHVkZV9zY3JlZW5zaG90OmZhbHNlfTsKZnVuY3Rpb24gcGFnZShyZXN1bHQsa2luZCkgewogIGNvbnN0IHM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudCxub2Rlcz1BcnJheS5pc0FycmF5KHM/LmNvbnRlbnRfcmVmcyk/cy5jb250ZW50X3JlZnM6W107CiAgY29uc3QgZmxhZ3M9ewogICAgc3RhdHVzX29rOnJlc3VsdD8uaXNFcnJvciE9PXRydWUmJnM/LnN0YXR1cz09PSJvayIsCiAgICBjb21wbGV0ZTpzPy5zbmFwc2hvdD8uY29tcGxldGU9PT10cnVlLAogICAgaGVhZGluZ19tYXRjaDpub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09CiAgICAgIChraW5kPT09InByb3RlY3RlZF9hZnRlciI/IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiOiJMb2NhbCBkZW1vIikpLAogICAgZXJyb3JfbWF0Y2g6bm9kZXMuc29tZShuPT5uLm5hbWU9PT0iTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LiIpLAogICAgcmVxdWlyZWRfdGV4dDpub2Rlcy5zb21lKG49Pm4ubmFtZT09PShraW5kPT09InByb3RlY3RlZF9iZWZvcmUiPyJTaWduIGluIHJlcXVpcmVkLiI6CiAgICAgIGtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIj8iU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iOgogICAgICAiVXNlIFNpZ24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLiIpKSwKICAgIGludGVyYWN0aXZlX3JlZl9wcmVzZW50OkFycmF5LmlzQXJyYXkocz8ucmVmcykmJgogICAgICBzLnJlZnMuc29tZShuPT5BcnJheS5pc0FycmF5KG4uYWN0aW9ucykmJm4uYWN0aW9ucy5pbmNsdWRlcygiY2xpY2siKSkKICB9OwogIHJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M9e2tpbmQsLi4uZmxhZ3N9O3JldGFpbigpOwogIC8vIE9ubHkgcHJvdmlkZXIgcmVmcyBhbmQgZml4ZWQgcHVibGljLWxhYmVsIG1hdGNoZXMgc3Vydml2ZSB0aGlzIHJhdyBzbmFwc2hvdC4KICBvd24uZnJlc2hfcmVmcz1BcnJheS5pc0FycmF5KHM/LnJlZnMpP3MucmVmcy5maWx0ZXIobj0+dHlwZW9mIG4ucmVmPT09InN0cmluZyImJgogICAgL15wWzAtOV0rOlswLTldKyQvLnRlc3Qobi5yZWYpKS5tYXAobj0+bi5yZWYpOltdOwogIHN0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7CiAgaWYoZmxhZ3MuZXJyb3JfbWF0Y2gpcmV0dXJuIGZhbHNlOwogIGlmKCFmbGFncy5zdGF0dXNfb2t8fCFmbGFncy5jb21wbGV0ZSlyZXR1cm4gZmFsc2U7CiAgaWYoa2luZD09PSJnZW5lcmljX2NoZWNrZWQiKSB7CiAgICBpZihub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiKSYmCiAgICAgICBub2Rlcy5zb21lKG49Pm4ubmFtZT09PSJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZhaWxhYmxlLiIpKQogICAgICByZWNvcmQucHJvdGVjdGVkX2FmdGVyX3BhZ2U9dHJ1ZTsKICAgIHJldHVybiB0cnVlOwogIH0KICBjb25zdCBvaz1mbGFncy5oZWFkaW5nX21hdGNoJiZmbGFncy5yZXF1aXJlZF90ZXh0JiYKICAgIChraW5kIT09ImFwcGxpY2F0aW9uInx8ZmxhZ3MuaW50ZXJhY3RpdmVfcmVmX3ByZXNlbnQpOwogIGlmKG9rJiZraW5kPT09InByb3RlY3RlZF9hZnRlciIpcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlPXRydWU7CiAgcmV0dXJuIG9rOwp9CmFzeW5jIGZ1bmN0aW9uIG5hdmlnYXRlQW5kU25hcHNob3QodXJsLGtpbmQpIHsKICBpZihhd2FpdCBjaGVja2VkKCJicm93c2VyX25hdmlnYXRpb24iLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19icm93c2VyX25hdmlnYXRlKHtzZXNzaW9uOm93bi5zZXNzaW9uLAogICAgICAgIHRhcmdldF9pZDpvd24udGFyZ2V0X2lkLHRhYl9pZDpvd24udGFiX2lkLHVybH0pLAogICAgICByPT5yPy5pc0Vycm9yIT09dHJ1ZSkpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLHI9PnBhZ2UocixraW5kKSk7CiAgfQp9CmFzeW5jIGZ1bmN0aW9uIGVudHJ5KCkgewogIC8vIFRoZSBleGFjdCBEcml2ZXItb3duZWQgYmxhbmsgaGFuZGxlcyBhbHJlYWR5IGV4aXN0IHdoaWxlIHRoZTE4MHMgZ2F0ZSB3YWl0cy4KICAvLyBUaGVyZSBpcyBubyBtb2RlbCB5aWVsZCwgdG9vbC1kZXNjcmlwdGlvbiBsb2FkIG9yIHJlYmluZCBhZnRlciB0aGlzIG1hcmtlci4KICBjb25zdCBtYXJrZXJDb21tYW5kPSJweXRob24zIC1jICIrc3EoTUFSS0VSKSsiICIrc3Eob3duLmxhYikrIiAiKwogICAgc3Eob3duLmd1YXJkX3BpZCkrIiAiK3NxKG93bi5zZXJ2ZXJfcGlkKTsKICBhd2FpdCBjaGVja2VkKCJwcmVwYXJlZF9tYXJrZXIiLAogICAgKCk9PnRvb2xzLmV4ZWNfY29tbWFuZCh7Y21kOm1hcmtlckNvbW1hbmQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgICB5aWVsZF90aW1lX21zOjEwMDAwLG1heF9vdXRwdXRfdG9rZW5zOjUwMH0pLHI9PnsKICAgICAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7bGFiZWw6Im1hcmtlciIsCiAgICAgICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgICAgIHNlc3Npb25faWQ6dHlwZW9mIHIuc2Vzc2lvbl9pZD09PSJudW1iZXIiP3Iuc2Vzc2lvbl9pZDpudWxsfSk7cmV0YWluKCk7CiAgICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgICAgcmV0dXJuIHIuZXhpdF9jb2RlPT09MCYmci5zZXNzaW9uX2lkPT09dW5kZWZpbmVkJiYKICAgICAgICBKU09OLnBhcnNlKHIub3V0cHV0KS5wcmVwYXJlZF9tYXJrZXJfd3JpdHRlbj09PXRydWU7CiAgICB9KTsKICBjb25zdCByZWFkeURlYWRsaW5lPURhdGUubm93KCkrMzAwMDA7CiAgd2hpbGUocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUmJkRhdGUubm93KCk8cmVhZHlEZWFkbGluZSkKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUpCiAgICBsYXRjaCgiZml4dHVyZV9yZWFkeV91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOyAvLyBPTkUgaW1tZWRpYXRlIHByZS1uYXZpZ2F0aW9uIHBvbGwsIG5vIFJQIEhUVFAgcHJvYmUuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBuYXZpZ2F0ZUFuZFNuYXBzaG90KCJodHRwOi8vbG9jYWxob3N0OjMwMDAvcHJvdGVjdGVkIiwicHJvdGVjdGVkX2JlZm9yZSIpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbClhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgYXdhaXQgbmF2aWdhdGVBbmRTbmFwc2hvdCgiaHR0cDovL2xvY2FsaG9zdDozMDAwLyIsImFwcGxpY2F0aW9uIik7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIE9ORSBwb3N0LWVudHJ5IHBvbGwuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICBvd24ucGhhc2U9ImJyb3dzZXJfZGVjaXNpb24iOwogICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfQphc3luYyBmdW5jdGlvbiBjb250aW51YXRpb24oKSB7CiAgY29uc3Qgc3BlYz1vd24ubmV4dF9kZWNpc2lvbjsKICBjb25zdCBhbGxvd2VkS2luZHM9WyJjbGljayIsInVzZXJuYW1lIiwicGFzc3dvcmQiLCJzbmFwc2hvdCIsInByb3RlY3RlZF9hZnRlciJdOwogIGlmKCFzcGVjfHwhYWxsb3dlZEtpbmRzLmluY2x1ZGVzKHNwZWMua2luZCkpIHsKICAgIGxhdGNoKCJicm93c2VyX2RlY2lzaW9uX3VuY29uZmlybWVkIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuOwogIH0KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgaWYoc3BlYy5raW5kPT09InByb3RlY3RlZF9hZnRlciIpIHsKICAgIC8vIFJlYWQgdGhlIGZyZXNoIGNhbGxiYWNrLWZvbGxvd2luZyBwcm90ZWN0ZWQgcGFnZTsgZG8gbm90IHNlbmQgYW5vdGhlciBSUCByZXF1ZXN0LgogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncykscj0+cGFnZShyLCJwcm90ZWN0ZWRfYWZ0ZXIiKSk7CiAgfSBlbHNlIGlmKHNwZWMua2luZD09PSJzbmFwc2hvdCIpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLAogICAgICByPT5wYWdlKHIsImdlbmVyaWNfY2hlY2tlZCIpJiZwdWJsaWNQcmVkaWNhdGUocixzcGVjKSk7CiAgfSBlbHNlIHsKICAgIGlmKHR5cGVvZiBzcGVjLnJlZiE9PSJzdHJpbmcifHwhQXJyYXkuaXNBcnJheShvd24uZnJlc2hfcmVmcyl8fAogICAgICAgIW93bi5mcmVzaF9yZWZzLmluY2x1ZGVzKHNwZWMucmVmKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIG93bi5mcmVzaF9yZWZzPVtdO3N0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7IC8vIFNpbmdsZS11c2Ugc25hcHNob3QgcmVmLgogICAgY29uc3QgYXJncz17c2Vzc2lvbjpvd24uc2Vzc2lvbix0YXJnZXRfaWQ6b3duLnRhcmdldF9pZCx0YWJfaWQ6b3duLnRhYl9pZCxyZWY6c3BlYy5yZWZ9OwogICAgY29uc3Qgb3BlcmF0aW9uPXNwZWMua2luZD09PSJjbGljayI/CiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2Jyb3dzZXJfY2xpY2soey4uLmFyZ3MsaW5wdXRfcm91dGU6ImRvbV9ldmVudCJ9KToKICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fYnJvd3Nlcl90eXBlKHsuLi5hcmdzLHJlcGxhY2U6dHJ1ZSwKICAgICAgICB0ZXh0OnNwZWMua2luZD09PSJ1c2VybmFtZSI/ImFkbWluIjpsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKX0pOwogICAgaWYoc3BlYy5raW5kPT09InBhc3N3b3JkIiYmKHR5cGVvZiBsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKSE9PSJzdHJpbmcifHwKICAgICAgICEvXltBLVphLXowLTlfLV17MzAsMTI4fSQvLnRlc3QobG9hZCgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IikpKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfaW5wdXQiLG9wZXJhdGlvbixyPT57CiAgICAgIGNvbnN0IHM9cj8uc3RydWN0dXJlZENvbnRlbnQ7CiAgICAgIHJldHVybiByPy5pc0Vycm9yIT09dHJ1ZSYmWyJjb25maXJtZWQiLCJ1bnZlcmlmaWFibGUiXS5pbmNsdWRlcyhzPy5lZmZlY3QpOwogICAgfSk7IC8vIERpc3BhdGNoIGFsb25lIG5ldmVyIGVhcm5zIGFuIGFwcGxpY2F0aW9uIG91dGNvbWUgb3Igam91cm5leSBjcmVkaXQuCiAgICBpZihzcGVjLmtpbmQ9PT0icGFzc3dvcmQiKXN0b3JlKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiLG51bGwpOwogICAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKQogICAgICBhd2FpdCBjaGVja2VkKCJicm93c2VyX3NuYXBzaG90IiwKICAgICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLAogICAgICAgIHI9PnBhZ2UociwiZ2VuZXJpY19jaGVja2VkIikmJnB1YmxpY1ByZWRpY2F0ZShyLHNwZWMpKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKICBlbHNlIGlmKHNwZWMua2luZD09PSJwcm90ZWN0ZWRfYWZ0ZXIiKSB7CiAgICByZWNvcmQuY2xlYW51cF9yZXF1ZXN0ZWRfd2FsbF9tcz1EYXRlLm5vdygpO3JldGFpbigpOwogICAgYXdhaXQgY2xlYW51cCgpOwogIH0KfQpmdW5jdGlvbiBwdWJsaWNQcmVkaWNhdGUocmVzdWx0LHNwZWMpIHsKICBjb25zdCBub2Rlcz1yZXN1bHQ/LnN0cnVjdHVyZWRDb250ZW50Py5jb250ZW50X3JlZnM7CiAgLy8gUm9vdCBtdXN0IHBpbiBvbmUgcHJpbnRlZCBwdWJsaWMgZXhwZWN0ZWQgbGFiZWwvcm9sZSBiZWZvcmUgdGhhdCBhY3Rpb24uCiAgLy8gTm8gcmF3IGZpZWxkIHZhbHVlLCBVUkwsIHN1YmplY3QsIHF1ZXJ5IG9yIGFyYml0cmFyeSByZWdleCBwcmVkaWNhdGUgaXMgYWxsb3dlZC4KICBjb25zdCByb2xlcz1bImhlYWRpbmciLCJidXR0b24iLCJzdGF0aWN0ZXh0IiwidGV4dGJveCJdOwogIGNvbnN0IGxhYmVscz1bIlVzZXJuYW1lIiwiUGFzc3dvcmQiLCJBdXRoZW50aWNhdG9yIG9yIHJlY292ZXJ5IGNvZGUiLCJTaWduIGluIiwKICAgICJMb2NhbCBkZW1vIiwiUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyIsCiAgICAiU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iXTsKICByZXR1cm4gQXJyYXkuaXNBcnJheShub2RlcykmJnJvbGVzLmluY2x1ZGVzKHNwZWMuZXhwZWN0ZWRfcm9sZSkmJgogICAgbGFiZWxzLmluY2x1ZGVzKHNwZWMuZXhwZWN0ZWRfbGFiZWwpJiYKICAgIG5vZGVzLnNvbWUobj0+bi5yb2xlPT09c3BlYy5leHBlY3RlZF9yb2xlJiZuLm5hbWU9PT1zcGVjLmV4cGVjdGVkX2xhYmVsKTsKfQp0cnkgewogIGlmKG93bi5waGFzZT09PSJwcmVwYXJlZF9lbnRyeSIpYXdhaXQgZW50cnkoKTtlbHNlIGF3YWl0IGNvbnRpbnVhdGlvbigpOwogIC8vIERvIG5vdCBjYXJyeSB1bnZhbGlkYXRlZCBwYXJ0aWFsIGNvbnRyb2xsZXIgdGV4dCBhY3Jvc3MgdGhpcyBjZWxsIGJvdW5kYXJ5LgogIHdoaWxlKGNvbnRyb2xsZXJCdWZmZXIubGVuZ3RoPjAmJnJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbCkgewogICAgY29uc3QgYmVmb3JlUG9sbFdhbGw9RGF0ZS5ub3coKTsKICAgIGlmKGJlZm9yZVBvbGxXYWxsPj1vd24uc3RhcnRfZXBvY2hfbXMrODQwMDAwKSB7CiAgICAgIGxhdGNoKCJicm93c2VyX2FjdGl2ZV9kZWFkbGluZSIsYmVmb3JlUG9sbFdhbGwpO2JyZWFrOwogICAgfQogICAgaWYoY29udHJvbGxlckpvaW5lZHx8IWNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlKSB7CiAgICAgIGxhdGNoKCJjb250cm9sbGVyX29ic2VydmF0aW9uX2ludmFsaWQiLGJlZm9yZVBvbGxXYWxsKTticmVhazsKICAgIH0KICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIFNhbWUgb3duZWQgc2Vzc2lvbjsgb2JzZXJ2ZXIgcmV0YWlucyByZWNlaXB0IGZpcnN0LgogICAgY29uc3QgYWZ0ZXJQb2xsV2FsbD1EYXRlLm5vdygpOwogICAgaWYoYWZ0ZXJQb2xsV2FsbD49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkKICAgICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixhZnRlclBvbGxXYWxsKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfSBjYXRjaCB7CiAgbGF0Y2goImNvbXBvc2VkX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7CiAgYXdhaXQgY2xlYW51cCgpOwp9CmlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCkgewogIHRleHQoe3Jlc3VsdDoiZmFpbGVkIixmaXJzdF9mYWlsdXJlOnJlY29yZC5maXJzdF9mYWlsdXJlLAogICAgdW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uOnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sCiAgICBkaWFnbm9zdGljX2Vycm9yczpyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMsY2xlYW51cF9lcnJvcnM6cmVjb3JkLmNsZWFudXBfZXJyb3JzLAogICAgZmluYWxfYWJzZW5jZTpyZWNvcmQuZmluYWxfYWJzZW5jZSxjb250cm9sbGVyX2V4aXQ6cmVjb3JkLmNvbnRyb2xsZXJfZXhpdCwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogICAgcmVzb3VyY2VfcmVsZWFzZV9wcm92ZW46cmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKX0pOwp9IGVsc2UgewogIHRleHQoe3Jlc3VsdDoiYnJvd3Nlcl9kZWNpc2lvbl9vYnNlcnZlZF9vbmx5IixwYWdlX2ZsYWdzOnJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M/P251bGwsCiAgICBqb3VybmV5X2NyZWRpdDpmYWxzZSxjbGVhbnVwX2Vycm9yczpyZWNvcmQuY2xlYW51cF9lcnJvcnMsCiAgICByZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpyZWNvcmQuY2xlYW51cF9zdGFydGVkPT09dHJ1ZSYmcmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKSwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlfSk7Cn0K",
  "baseline_b64": "Ly8gUHJvc3BlY3RpdmUgT05FIGZ1bmN0aW9ucy5leGVjIGNlbGw7IE5PVCBleGVjdXRlZCBpbiB0aGlzIGRlc2lnbiBwaGFzZS4KY29uc3QgQ0xPQ0s9ImltcG9ydCBqc29uLHRpbWVcbnByaW50KGpzb24uZHVtcHMoeydtb25vdG9uaWNfbnMnOnN0cih0aW1lLm1vbm90b25pY19ucygpKSwnd2FsbF9lcG9jaF9ucyc6c3RyKHRpbWUudGltZV9ucygpKX0pKVxuIjsKY29uc3QgU1RPUD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzdGF0LHN5cyx0aW1lXG5jPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMV0pXG5jbG9jaz17J21vbm90b25pY19ucyc6c3RyKHRpbWUubW9ub3RvbmljX25zKCkpLCd3YWxsX2Vwb2NoX25zJzpzdHIodGltZS50aW1lX25zKCkpfVxucmVjb3JkPXsnc2NoZW1hJzoncmlhdXRoLmQwMS1maXJzdC1vYnNlcnZhdGlvbi92MScsJ2ZpcnN0X2ZhaWx1cmUnOmNbJ2ZpcnN0X2ZhaWx1cmUnXSwnZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyc6Y1snZmlyc3Rfb2JzZXJ2YXRpb25fd2FsbF9tcyddLCdmaXJzdF9ldmVudF9wcm92ZW4nOkZhbHNlLCdjbG9jayc6Y2xvY2t9XG5ldmVudF9zdGF0ZT0nd3JpdGVfdW5jb25maXJtZWQnXG50cnk6XG4gICAgZmQ9b3Mub3BlbihwYXRobGliLlBhdGgoY1snZXZlbnRfb3V0J10pLG9zLk9fV1JPTkxZfG9zLk9fQ1JFQVR8b3MuT19FWENMfG9zLk9fTk9GT0xMT1csMG82MDApXG4gICAgd2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgICAgIGYud3JpdGUoanNvbi5kdW1wcyhyZWNvcmQsc29ydF9rZXlzPVRydWUpKydcXG4nKTtmLmZsdXNoKCk7b3MuZnN5bmMoZi5maWxlbm8oKSlcbiAgICBldmVudF9zdGF0ZT0nd3JpdHRlbidcbmV4Y2VwdCBPU0Vycm9yOnBhc3NcbiMgQXR0ZW1wdCBjbG9jayBwZXJzaXN0ZW5jZSBCRUZPUkUgbWFya2VyL3N0YXRlL2J1ZGdldCBjb21wYXJpc29ucy5cbiMgQSBtZXRhZGF0YSBlcnJvciBkb2VzIG5vdCBwcmV2ZW50IHRoZSBvcmlnaW5hbCBlc3NlbnRpYWwgc3RvcCBwcm90b2NvbC5cbmxhYj1wYXRobGliLlBhdGgoY1snbGFiJ10pO21hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbnRyeTpcbiAgICBpZiBsYWIuZXhpc3RzKCk6XG4gICAgICAgIGluZm89bGFiLmxzdGF0KClcbiAgICAgICAgaWYgbm90IHN0YXQuU19JU0RJUihpbmZvLnN0X21vZGUpIG9yIHN0YXQuU19JTU9ERShpbmZvLnN0X21vZGUpIT0wbzcwMCBvciBpbmZvLnN0X3VpZCE9b3MuZ2V0dWlkKCk6XG4gICAgICAgICAgICBtYXJrZXJfc3RhdGU9J293bmVyc2hpcF91bmtub3duJ1xuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgbWFya2VyX3N0YXRlPSdzdG9wX3JlcXVlc3RlZCdcbiAgICAgICAgICAgIGZvciBuYW1lIGluICgoJ3VpLWZhaWx1cmUnLCdzdG9wJykgaWYgY1snZmlyc3RfZmFpbHVyZSddIGlzIG5vdCBOb25lIGVsc2UgKCdzdG9wJywpKTpcbiAgICAgICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgICAgIGZkPW9zLm9wZW4obGFiL25hbWUsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbiAgICAgICAgICAgICAgICAgICAgb3MuY2xvc2UoZmQpXG4gICAgICAgICAgICAgICAgZXhjZXB0IEZpbGVOb3RGb3VuZEVycm9yOm1hcmtlcl9zdGF0ZT0nbGFiX2Fic2VudCdcbiAgICAgICAgICAgICAgICBleGNlcHQgRmlsZUV4aXN0c0Vycm9yOnBhc3NcbmV4Y2VwdCBPU0Vycm9yOm1hcmtlcl9zdGF0ZT0nc3RvcF91bmNvbmZpcm1lZCdcbnByaW50KGpzb24uZHVtcHMoeydjbG9jayc6Y2xvY2ssJ2V2ZW50X3N0YXRlJzpldmVudF9zdGF0ZSwnbWFya2VyX3N0YXRlJzptYXJrZXJfc3RhdGV9KSlcbiI7CmNvbnN0IFJFQURCQUNLPSJpbXBvcnQganNvbixvcyxwYXRobGliLHN0YXQsc3VicHJvY2VzcyxzeXMsdGltZVxuYz1qc29uLmxvYWRzKHN5cy5hcmd2WzFdKVxuY2hpbGRyZW49Tm9uZTtoZWxwZXJfcGlkPWNbJ2hlbHBlcl9waWQnXTtvdXRlcl9vYnNlcnZhdGlvbj1Ob25lO3Byb2plY3Rpb249J3VuYXZhaWxhYmxlJ1xuZGVmIGZpbml0ZV9vYnNlcnZhdGlvbih2YWx1ZSk6XG4gICAgaWYgdHlwZSh2YWx1ZSkgaXMgbm90IGRpY3Qgb3Igc2V0KHZhbHVlKSE9IHsnb2JzZXJ2ZWQnLCdkaWFnbm9zdGljJ30gb3IgdHlwZSh2YWx1ZVsnb2JzZXJ2ZWQnXSkgaXMgbm90IGJvb2w6XG4gICAgICAgIHJldHVybiBOb25lXG4gICAgZGlhZ25vc3RpYz12YWx1ZVsnZGlhZ25vc3RpYyddXG4gICAgaWYgZGlhZ25vc3RpYyBpcyBOb25lOlxuICAgICAgICByZXR1cm4geydvYnNlcnZlZCc6dmFsdWVbJ29ic2VydmVkJ10sJ2RpYWdub3N0aWMnOk5vbmV9XG4gICAgaWYgbm90IHZhbHVlWydvYnNlcnZlZCddIG9yIHR5cGUoZGlhZ25vc3RpYykgaXMgbm90IGRpY3Qgb3Igc2V0KGRpYWdub3N0aWMpIT0geydzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnfTpcbiAgICAgICAgcmV0dXJuIE5vbmVcbiAgICBzaXRlcz0oJ2hhbmRsZXInLCdzZXJ2ZXInLCdtYWluJylcbiAgICBraW5kcz0oJ0F0dHJpYnV0ZUVycm9yJywnVHlwZUVycm9yJywnVmFsdWVFcnJvcicsJ0tleUVycm9yJywnT1NFcnJvcicsJ0Jyb2tlblBpcGVFcnJvcicsJ0Nvbm5lY3Rpb25SZXNldEVycm9yJywnVGltZW91dEVycm9yJywnb3RoZXInKVxuICAgIGZ1bmN0aW9ucz0oJ0hlYWRlclJlYWRlci5yZWFkbGluZScsJ0RlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0JywnRGVtby5iZWdpbicsJ0RlbW8uY2FsbGJhY2snLCdEZW1vLmludm9rZScsJ0hhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0JywnSGFuZGxlci5zZW5kX2Vycm9yJywnSGFuZGxlci5nZXQnLCdIYW5kbGVyLnJlcGx5JywnbWFpbicpXG4gICAgc2l0ZSxraW5kLGZ1bmN0aW9uLGxpbmU9KGRpYWdub3N0aWNba10gZm9yIGsgaW4gKCdzaXRlJywnZXhjZXB0aW9uX2NsYXNzJywnb3duX2Z1bmN0aW9uJywnb3duX2xpbmUnKSlcbiAgICBpZiB0eXBlKHNpdGUpIGlzIG5vdCBzdHIgb3Igc2l0ZSBub3QgaW4gc2l0ZXMgb3IgdHlwZShraW5kKSBpcyBub3Qgc3RyIG9yIGtpbmQgbm90IGluIGtpbmRzOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIGlmIG5vdCAoKGZ1bmN0aW9uIGlzIE5vbmUgYW5kIGxpbmUgaXMgTm9uZSkgb3IgKHR5cGUoZnVuY3Rpb24pIGlzIHN0ciBhbmQgZnVuY3Rpb24gaW4gZnVuY3Rpb25zIGFuZCB0eXBlKGxpbmUpIGlzIGludCBhbmQgMTw9bGluZTw9MTAyNCkpOlxuICAgICAgICByZXR1cm4gTm9uZVxuICAgIHJldHVybiB7J29ic2VydmVkJzpUcnVlLCdkaWFnbm9zdGljJzp7J3NpdGUnOnNpdGUsJ2V4Y2VwdGlvbl9jbGFzcyc6a2luZCwnb3duX2Z1bmN0aW9uJzpmdW5jdGlvbiwnb3duX2xpbmUnOmxpbmV9fVxub3V0ZXI9cGF0aGxpYi5QYXRoKGNbJ291dGVyX291dCddKVxuaWYgb3V0ZXIuaXNfZmlsZSgpOlxuICAgIGZkPW9zLm9wZW4ob3V0ZXIsb3MuT19SRE9OTFl8b3MuT19OT0ZPTExPV3xvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbz1vcy5mc3RhdChmZClcbiAgICAgICAgaWYgbm90IChzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQ9PW9zLmdldHVpZCgpIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKT09MG82MDAgYW5kIDA8aW5mby5zdF9zaXplPD0yNjIxNDQpOlxuICAgICAgICAgICAgcmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIHdpdGggb3MuZmRvcGVuKGZkLCdyYicsY2xvc2VmZD1GYWxzZSkgYXMgc291cmNlOnJhd19ieXRlcz1zb3VyY2UucmVhZCgyNjIxNDUpXG4gICAgICAgIGlmIG5vdCAwPGxlbihyYXdfYnl0ZXMpPD0yNjIxNDQ6cmFpc2UgVmFsdWVFcnJvcignZml4ZWRfb3V0ZXJfZXZpZGVuY2VfaW52YWxpZCcpXG4gICAgICAgIGRhdGE9anNvbi5sb2FkcyhyYXdfYnl0ZXMpXG4gICAgZmluYWxseTpvcy5jbG9zZShmZClcbiAgICBvdXRlcl9vYnNlcnZhdGlvbj1maW5pdGVfb2JzZXJ2YXRpb24oZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbicpKVxuICAgIHByb2plY3Rpb249ZGF0YS5nZXQoJ3VuZXhwZWN0ZWRfb2JzZXJ2YXRpb25fcHJvamVjdGlvbicpXG4gICAgaWYgcHJvamVjdGlvbiBub3QgaW4gKCd1bmF2YWlsYWJsZScsJ3ZhbGlkJywnaW52YWxpZCcpOnByb2plY3Rpb249J2ludmFsaWQnXG4gICAgaWYgcHJvamVjdGlvbj09J3ZhbGlkJyBhbmQgb3V0ZXJfb2JzZXJ2YXRpb24gaXMgTm9uZTpwcm9qZWN0aW9uPSdpbnZhbGlkJ1xuICAgIGFsbG93ZWQ9eyd3aG9hbWknLCdkaXNjb3ZlcnknLCdjb25maWRlbnRpYWxfY2xpZW50X2NyZWF0ZScsJ29wZXJhdG9yX2xvZ2luJywnc2VydmVyJywnbWFpbnRlbmFuY2VfaW5pdCd9XG4gICAgc3RhcnRlZD1kYXRhLmdldCgnaGVscGVyX2ludm9jYXRpb25zJyk9PTEgYW5kIHR5cGUoZGF0YS5nZXQoJ2hlbHBlcl9waWQnKSkgaXMgaW50IGFuZCBkYXRhWydoZWxwZXJfcGlkJ10+MFxuICAgIGlmIHN0YXJ0ZWQ6YWxsb3dlZC5hZGQoJ2hlbHBlcicpXG4gICAgcmF3PWRhdGEuZ2V0KCdvd25lZF9jaGlsZF9leGl0cycpXG4gICAgaWYgaXNpbnN0YW5jZShyYXcsbGlzdCkgYW5kIGxlbihyYXcpPT1sZW4oYWxsb3dlZCkgYW5kIGFsbChpc2luc3RhbmNlKHYsZGljdCkgYW5kIHYuZ2V0KCduYW1lJykgaW4gYWxsb3dlZCBhbmQgdHlwZSh2LmdldCgncGlkJykpIGlzIGludCBhbmQgdlsncGlkJ10+MCBhbmQgdHlwZSh2LmdldCgnZXhpdCcpKSBpcyBpbnQgZm9yIHYgaW4gcmF3KSBhbmQge3ZbJ25hbWUnXSBmb3IgdiBpbiByYXd9PT1hbGxvd2VkIGFuZCBsZW4oe3ZbJ3BpZCddIGZvciB2IGluIHJhd30pPT1sZW4oYWxsb3dlZCkgYW5kIG5leHQodlsncGlkJ10gZm9yIHYgaW4gcmF3IGlmIHZbJ25hbWUnXT09J3NlcnZlcicpPT1jWydzZXJ2ZXJfcGlkJ10gYW5kICgoc3RhcnRlZCBhbmQgbmV4dCh2WydwaWQnXSBmb3IgdiBpbiByYXcgaWYgdlsnbmFtZSddPT0naGVscGVyJyk9PWRhdGFbJ2hlbHBlcl9waWQnXSBhbmQgKGhlbHBlcl9waWQgaXMgTm9uZSBvciBoZWxwZXJfcGlkPT1kYXRhWydoZWxwZXJfcGlkJ10pKSBvciAobm90IHN0YXJ0ZWQgYW5kIGhlbHBlcl9waWQgaXMgTm9uZSBhbmQgZGF0YS5nZXQoJ2hlbHBlcl9pbnZvY2F0aW9ucycpIGlzIE5vbmUgYW5kIGRhdGEuZ2V0KCdoZWxwZXJfcGlkJykgaXMgTm9uZSkpOlxuICAgICAgICBjaGlsZHJlbj1be2s6dltrXSBmb3IgayBpbiAoJ25hbWUnLCdwaWQnLCdleGl0Jyl9IGZvciB2IGluIHJhd11cbiAgICAgICAgaGVscGVyX3BpZD1kYXRhWydoZWxwZXJfcGlkJ10gaWYgc3RhcnRlZCBlbHNlIE5vbmVcbnBpZHM9bGlzdChjWydvd25lZF9waWRzJ10pXG5pZiBoZWxwZXJfcGlkIGlzIG5vdCBOb25lIGFuZCBoZWxwZXJfcGlkIG5vdCBpbiBwaWRzOnBpZHMuYXBwZW5kKGhlbHBlcl9waWQpXG5wPXN1YnByb2Nlc3MucnVuKFsnL2Jpbi9wcycsJy1wJywnLCcuam9pbihzdHIodikgZm9yIHYgaW4gcGlkcyksJy1vJywncGlkPSddLGNhcHR1cmVfb3V0cHV0PVRydWUsdGltZW91dD0zKVxucHNfa25vd249cC5yZXR1cm5jb2RlIGluICgwLDEpIGFuZCBhbGwodi5pc2RpZ2l0KCkgZm9yIHYgaW4gcC5zdGRvdXQuc3BsaXQoKSlcbnByZXNlbnQ9c2V0KGludCh2KSBmb3IgdiBpbiBwLnN0ZG91dC5zcGxpdCgpKSBpZiBwc19rbm93biBlbHNlIHNldCgpXG5wb3J0cz17fVxuZm9yIHBvcnQgaW4gKDkwMDAsMzAwMCk6XG4gICAgcD1zdWJwcm9jZXNzLnJ1bihbJy91c3Ivc2Jpbi9sc29mJywnLW5QJywnLXQnLCctaVRDUDonK3N0cihwb3J0KSwnLXNUQ1A6TElTVEVOJ10sY2FwdHVyZV9vdXRwdXQ9VHJ1ZSx0aW1lb3V0PTMpXG4gICAgcG9ydHNbc3RyKHBvcnQpXT1ub3QgYm9vbChwLnN0ZG91dC5zdHJpcCgpKSBpZiBwLnJldHVybmNvZGUgaW4gKDAsMSkgZWxzZSBOb25lXG5wcmludChqc29uLmR1bXBzKHsnY2xvY2snOnsnbW9ub3RvbmljX25zJzpzdHIodGltZS5tb25vdG9uaWNfbnMoKSksJ3dhbGxfZXBvY2hfbnMnOnN0cih0aW1lLnRpbWVfbnMoKSl9LCdvd25lZF9waWRzJzp7c3RyKHYpOih2IG5vdCBpbiBwcmVzZW50IGlmIHBzX2tub3duIGVsc2UgTm9uZSkgZm9yIHYgaW4gcGlkc30sJ3BvcnRzJzpwb3J0cywnbGFiX2Fic2VudCc6bm90IHBhdGhsaWIuUGF0aChjWydsYWInXSkuZXhpc3RzKCksJ293bmVkX2NoaWxkX2V4aXRzJzpjaGlsZHJlbiwnaGVscGVyX3BpZCc6aGVscGVyX3BpZCwndW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uJzpvdXRlcl9vYnNlcnZhdGlvbiwndW5leHBlY3RlZF9vYnNlcnZhdGlvbl9wcm9qZWN0aW9uJzpwcm9qZWN0aW9ufSkpXG4iOwpjb25zdCBNQVJLRVI9ImltcG9ydCBvcyxwYXRobGliLHN0YXQsc3lzXG5sYWI9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKTtndWFyZD1pbnQoc3lzLmFyZ3ZbMl0pO3NlcnZlcj1pbnQoc3lzLmFyZ3ZbM10pXG5pZiBndWFyZDw9MCBvciBzZXJ2ZXI8PTAgb3IgZ3VhcmQ9PXNlcnZlcjpyYWlzZSBWYWx1ZUVycm9yKCdvd25lZF9jb250ZXh0X2ludmFsaWQnKVxuaW5mbz1sYWIubHN0YXQoKVxuaWYgbm90IChzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSk9PTBvNzAwIGFuZCBpbmZvLnN0X3VpZD09b3MuZ2V0dWlkKCkpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5vcy5raWxsKGd1YXJkLDApO29zLmtpbGwoc2VydmVyLDApXG5pZiAobGFiLydzdG9wJykuZXhpc3RzKCkgb3IgKGxhYi8ndWktZmFpbHVyZScpLmV4aXN0cygpOnJhaXNlIFZhbHVlRXJyb3IoJ293bmVkX2NvbnRleHRfaW52YWxpZCcpXG5mZD1vcy5vcGVuKGxhYi8nYnJvd3Nlci1wcmVwYXJlZCcsb3MuT19XUk9OTFl8b3MuT19DUkVBVHxvcy5PX0VYQ0x8b3MuT19OT0ZPTExPVywwbzYwMClcbm9zLmNsb3NlKGZkKVxucHJpbnQoJ3tcInByZXBhcmVkX21hcmtlcl93cml0dGVuXCI6dHJ1ZX0nKVxuIjsKY29uc3QgUEVSU0lTVD0iaW1wb3J0IGpzb24sb3MscGF0aGxpYixzeXNcbnBhdGg9cGF0aGxpYi5QYXRoKHN5cy5hcmd2WzFdKVxucmVjb3JkPWpzb24ubG9hZHMoc3lzLmFyZ3ZbMl0pXG4jIENvbnZlcnQgZGVjaW1hbCBzdHJpbmdzIGRpcmVjdGx5IHRvIFB5dGhvbiBpbnRlZ2VycywgYXZvaWRpbmcgSlMgTnVtYmVyIHJvdW5kaW5nLlxuZGVmIGNsb2Nrcyh2YWx1ZSk6XG4gICAgaWYgaXNpbnN0YW5jZSh2YWx1ZSxkaWN0KTpcbiAgICAgICAgZm9yIGssdiBpbiBsaXN0KHZhbHVlLml0ZW1zKCkpOlxuICAgICAgICAgICAgaWYgayBpbiAoJ21vbm90b25pY19ucycsJ3dhbGxfZXBvY2hfbnMnKSBhbmQgaXNpbnN0YW5jZSh2LHN0cik6XG4gICAgICAgICAgICAgICAgYXNzZXJ0IHYuaXNkZWNpbWFsKCkgYW5kIGxlbih2KTw9MTkgYW5kIDA8PWludCh2KTwyKio2M1xuICAgICAgICAgICAgICAgIHZhbHVlW2tdPWludCh2KVxuICAgICAgICAgICAgZWxzZTpjbG9ja3ModilcbiAgICBlbGlmIGlzaW5zdGFuY2UodmFsdWUsbGlzdCk6XG4gICAgICAgIGZvciB2IGluIHZhbHVlOmNsb2Nrcyh2KVxuY2xvY2tzKHJlY29yZClcbmZkPW9zLm9wZW4ocGF0aCxvcy5PX1dST05MWXxvcy5PX0NSRUFUfG9zLk9fRVhDTHxvcy5PX05PRk9MTE9XLDBvNjAwKVxud2l0aCBvcy5mZG9wZW4oZmQsJ3cnLGVuY29kaW5nPSdhc2NpaScpIGFzIGY6XG4gICAgZi53cml0ZShqc29uLmR1bXBzKHJlY29yZCxzb3J0X2tleXM9VHJ1ZSxpbmRlbnQ9MikrJ1xcbicpO2YuZmx1c2goKTtvcy5mc3luYyhmLmZpbGVubygpKVxucHJpbnQoanNvbi5kdW1wcyh7J3dyaXR0ZW5fZXhjbHVzaXZlJzpUcnVlfSkpXG4iOwpjb25zdCBvd249bG9hZCgiZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIik7CmNvbnN0IE9CU0tFWT0iZDAxX2ltbWVkaWF0ZV9vYnNlcnZhdGlvbl9yZWNvcmQiOwppZiAoIW93biB8fCBvd24ucnVudGltZV9yZWxlYXNlIT09dHJ1ZSB8fCBvd24uZHJpdmVyX293bmVkIT09dHJ1ZSB8fAogICAgb3duLmV4YWN0X2JvdW5kIT09dHJ1ZSB8fCBvd24uc2luZ2xlX2NvbnRyb2xsZXIhPT10cnVlIHx8CiAgICAhWyJwcmVwYXJlZF9lbnRyeSIsImJyb3dzZXJfZGVjaXNpb24iXS5pbmNsdWRlcyhvd24ucGhhc2UpIHx8CiAgICAob3duLnBoYXNlPT09InByZXBhcmVkX2VudHJ5IiYmKG93bi5wcmVwYXJlZF9nYXRlIT09dHJ1ZXx8b3duLmhlbHBlcl9waWQhPT1udWxsfHwKICAgICAgb3duLmZpeHR1cmVfcmVhZHk9PT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mPT09dHJ1ZSkpIHx8CiAgICAob3duLnBoYXNlPT09ImJyb3dzZXJfZGVjaXNpb24iJiYob3duLmZpeHR1cmVfcmVhZHkhPT10cnVlfHxvd24ubGlzdGVuZXJfcGlkX3Byb29mIT09dHJ1ZSkpIHx8CiAgICBvd24uZnJlc2hfcGF0aHNfcHJlZmxpZ2h0IT09dHJ1ZSB8fAogICAgIU51bWJlci5pc1NhZmVJbnRlZ2VyKG93bi5zdGFydF9lcG9jaF9tcykgfHwgdHlwZW9mIG93bi53b3Jrc3BhY2UhPT0ic3RyaW5nIiB8fAogICAgbmV3IFNldChbb3duLmd1YXJkX3BpZCxvd24uc2VydmVyX3BpZCxvd24uYnJvd3Nlcl9waWQsCiAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSkuc2l6ZSE9PShvd24uaGVscGVyX3BpZD09PW51bGw/Mzo0KSB8fAogICAgIVtvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCxvd24uZXhlY19zZXNzaW9uLAogICAgICAgLi4uKG93bi5oZWxwZXJfcGlkPT09bnVsbD9bXTpbb3duLmhlbHBlcl9waWRdKV0KICAgICAgIC5ldmVyeSh2PT5OdW1iZXIuaXNTYWZlSW50ZWdlcih2KSYmdj4wKSB8fAogICAgIVtvd24uc2Vzc2lvbixvd24udGFyZ2V0X2lkLG93bi50YWJfaWQsb3duLmxhYixvd24ub3V0ZXJfb3V0LAogICAgICAgb3duLmV2ZW50X291dCxvd24uY2xlYW51cF9vdXRdLmV2ZXJ5KHY9PnR5cGVvZiB2PT09InN0cmluZyImJnYubGVuZ3RoPjApKSB7CiAgdGV4dCh7cHJvcG9zYWxfcmVmdXNlZDoiZnJlc2hfb3duZWRfY29udGV4dF9yZXF1aXJlZCJ9KTtleGl0KCk7Cn0KY29uc3QgcHJpb3I9bG9hZChPQlNLRVkpOwppZihvd24uY2xvc2VkPT09dHJ1ZXx8cHJpb3I/LmNsZWFudXBfc3RhcnRlZD09PXRydWUpewogIHRleHQoe3Byb3Bvc2FsX3JlZnVzZWQ6ImZpeHR1cmVfYWxyZWFkeV9zdG9wcGVkIixyZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpmYWxzZX0pO2V4aXQoKTsKfQpjb25zdCByZWNvcmQ9cHJpb3I/P3sKICBzY2hlbWE6InJpYXV0aC5kMDEtaW1tZWRpYXRlLW9ic2VydmF0aW9uLWNsZWFudXAvdjEiLAogIGZpcnN0X2ZhaWx1cmU6bnVsbCxmaXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOm51bGwsCiAgZmlyc3RfZXZlbnRfcHJvdmVuOmZhbHNlLHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogIHVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbjpudWxsLGRpYWdub3N0aWNfZXJyb3JzOltdLGNsZWFudXBfc3RhcnRlZDpmYWxzZSwKICBmaXJzdF9jbG9jazpudWxsLG9ic2VydmF0aW9uX3JlY2VpcHRzOltdLGNvbnRyb2xsZXJfb2JzZXJ2YXRpb25zOltdLGxvY2FsX3Rvb2xfcmVjZWlwdHM6W10sYWN0aW9uczpbXSxjbGVhbnVwX2Vycm9yczpbXSwKICBmaW5hbF9hYnNlbmNlOm51bGwsb3duZWRfY2hpbGRfZXhpdHM6bnVsbCxjb250cm9sbGVyX2V4aXQ6bnVsbCwKICBmaXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zOm51bGwsbGF0Y2hfdG9fYWJzZW5jZV9tczpudWxsCn07CmNvbnN0IHNxPXM9PiInIitTdHJpbmcocykucmVwbGFjZSgvJy9nLCInXFwnJyIpKyInIjsKY29uc3QgcmV0YWluPSgpPT5zdG9yZShPQlNLRVkscmVjb3JkKTsKY29uc3QgY2xlYW51cEVycm9yPWxhYmVsPT57CiAgaWYoIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5pbmNsdWRlcyhsYWJlbCkpcmVjb3JkLmNsZWFudXBfZXJyb3JzLnB1c2gobGFiZWwpOwogIHJldGFpbigpOwp9Owpjb25zdCBsb2NhbD1hc3luYyhzb3VyY2UsYXJnKT0+ewogIGNvbnN0IGNtZD0icHl0aG9uMyAtYyAiK3NxKHNvdXJjZSkrKGFyZz09PXVuZGVmaW5lZD8iIjoiICIrc3EoSlNPTi5zdHJpbmdpZnkoYXJnKSkpOwogIGNvbnN0IHI9YXdhaXQgdG9vbHMuZXhlY19jb21tYW5kKHtjbWQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgeWllbGRfdGltZV9tczoxMDAwMCxtYXhfb3V0cHV0X3Rva2VuczoyMDAwfSk7CiAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7CiAgICBsYWJlbDpzb3VyY2U9PT1DTE9DSz8iY2xvY2siOnNvdXJjZT09PVNUT1A/InN0b3AiOnNvdXJjZT09PVJFQURCQUNLPyJyZWFkYmFjayI6InVua25vd24iLAogICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGwKICB9KTtyZXRhaW4oKTsgLy8gTnVtZXJpYyBjb2xsZWN0b3IgcmVjZWlwdCBCRUZPUkUgY29tcGFyaXNvbnMuCiAgaWYoci5zZXNzaW9uX2lkIT09dW5kZWZpbmVkKSB7CiAgICBjbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTt0aHJvdyBuZXcgRXJyb3IoImxvY2FsX3BlbmRpbmciKTsKICB9CiAgaWYoci5leGl0X2NvZGUhPT0wKXRocm93IG5ldyBFcnJvcigibG9jYWxfZmFpbGVkIik7CiAgcmV0dXJuIEpTT04ucGFyc2Uoci5vdXRwdXQpOwp9OwpmdW5jdGlvbiBmaW5pdGVPYnNlcnZhdGlvbih2YWx1ZSkgewogIGNvbnN0IGV4YWN0PSh2LGtleXMpPT52IT09bnVsbCYmdHlwZW9mIHY9PT0ib2JqZWN0IiYmIUFycmF5LmlzQXJyYXkodikmJgogICAgT2JqZWN0LmtleXModikubGVuZ3RoPT09a2V5cy5sZW5ndGgmJmtleXMuZXZlcnkoaz0+T2JqZWN0Lmhhc093bih2LGspKTsKICBpZighZXhhY3QodmFsdWUsWyJvYnNlcnZlZCIsImRpYWdub3N0aWMiXSl8fHR5cGVvZiB2YWx1ZS5vYnNlcnZlZCE9PSJib29sZWFuIilyZXR1cm4gbnVsbDsKICBjb25zdCBkPXZhbHVlLmRpYWdub3N0aWM7CiAgaWYoZD09PW51bGwpcmV0dXJuIHtvYnNlcnZlZDp2YWx1ZS5vYnNlcnZlZCxkaWFnbm9zdGljOm51bGx9OwogIGlmKCF2YWx1ZS5vYnNlcnZlZHx8IWV4YWN0KGQsWyJzaXRlIiwiZXhjZXB0aW9uX2NsYXNzIiwib3duX2Z1bmN0aW9uIiwib3duX2xpbmUiXSl8fAogICAgICFbImhhbmRsZXIiLCJzZXJ2ZXIiLCJtYWluIl0uaW5jbHVkZXMoZC5zaXRlKXx8CiAgICAgIVsiQXR0cmlidXRlRXJyb3IiLCJUeXBlRXJyb3IiLCJWYWx1ZUVycm9yIiwiS2V5RXJyb3IiLCJPU0Vycm9yIiwiQnJva2VuUGlwZUVycm9yIiwKICAgICAgICJDb25uZWN0aW9uUmVzZXRFcnJvciIsIlRpbWVvdXRFcnJvciIsIm90aGVyIl0uaW5jbHVkZXMoZC5leGNlcHRpb25fY2xhc3MpKXJldHVybiBudWxsOwogIGNvbnN0IGZ1bmN0aW9ucz1bIkhlYWRlclJlYWRlci5yZWFkbGluZSIsIkRlbW9TZXJ2ZXIucHJvY2Vzc19yZXF1ZXN0IiwiRGVtby5iZWdpbiIsIkRlbW8uY2FsbGJhY2siLAogICAgIkRlbW8uaW52b2tlIiwiSGFuZGxlci5oYW5kbGVfb25lX3JlcXVlc3QiLCJIYW5kbGVyLnNlbmRfZXJyb3IiLCJIYW5kbGVyLmdldCIsIkhhbmRsZXIucmVwbHkiLCJtYWluIl07CiAgaWYoISgoZC5vd25fZnVuY3Rpb249PT1udWxsJiZkLm93bl9saW5lPT09bnVsbCl8fAogICAgICAgKGZ1bmN0aW9ucy5pbmNsdWRlcyhkLm93bl9mdW5jdGlvbikmJk51bWJlci5pc1NhZmVJbnRlZ2VyKGQub3duX2xpbmUpJiYKICAgICAgICBkLm93bl9saW5lPj0xJiZkLm93bl9saW5lPD0xMDI0KSkpcmV0dXJuIG51bGw7CiAgcmV0dXJuIHtvYnNlcnZlZDp0cnVlLGRpYWdub3N0aWM6e3NpdGU6ZC5zaXRlLGV4Y2VwdGlvbl9jbGFzczpkLmV4Y2VwdGlvbl9jbGFzcywKICAgIG93bl9mdW5jdGlvbjpkLm93bl9mdW5jdGlvbixvd25fbGluZTpkLm93bl9saW5lfX07Cn0KZnVuY3Rpb24gZGlhZ25vc3RpYyh2YWx1ZSxwcm9qZWN0aW9uKSB7CiAgY29uc3QgcHJvamVjdGVkPWZpbml0ZU9ic2VydmF0aW9uKHZhbHVlKTsKICBpZihwcm9qZWN0aW9uPT09ImludmFsaWQifHwhWyJ2YWxpZCIsInVuYXZhaWxhYmxlIl0uaW5jbHVkZXMocHJvamVjdGlvbil8fAogICAgIChwcm9qZWN0aW9uPT09InZhbGlkIiYmcHJvamVjdGVkPT09bnVsbCkpIHsKICAgIGlmKCFyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMuaW5jbHVkZXMoIm9ic2VydmVyX3Byb2plY3Rpb25faW52YWxpZCIpKQogICAgICByZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMucHVzaCgib2JzZXJ2ZXJfcHJvamVjdGlvbl9pbnZhbGlkIik7CiAgfQogIGlmKHByb2plY3RlZCE9PW51bGwmJnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb249PT1udWxsKQogICAgcmVjb3JkLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbj1wcm9qZWN0ZWQ7CiAgcmV0YWluKCk7IC8vIE5vIHJhdyBkaWFnbm9zdGljIGZhbGxiYWNrIGFuZCBubyBmaXJzdC1mYWlsdXJlIG9yIG91dGNvbWUgbXV0YXRpb24uCn0KY29uc3QgbnM9Yz0+QmlnSW50KGMubW9ub3RvbmljX25zKTsKY29uc3QgZ2V0Q2xvY2s9KCk9PmxvY2FsKENMT0NLKTsKbGV0IGNvbnRyb2xsZXJKb2luZWQ9ZmFsc2U7CmxldCBjb250cm9sbGVyUG9sbEF2YWlsYWJsZT10cnVlOwpsZXQgY29udHJvbGxlckJ1ZmZlcj0iIjsKbGV0IGNsZWFudXBTdGFydGVkPWZhbHNlOwpsZXQgbGFzdFNuYXBzaG90RmxhZ3M9bnVsbDsKZnVuY3Rpb24gbGF0Y2gobGFiZWwscmVjZWl2ZWRXYWxsKSB7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICByZWNvcmQuZmlyc3RfZmFpbHVyZT1sYWJlbDsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPXJlY2VpdmVkV2FsbDsKICAgIHJldGFpbigpOyAvLyBTeW5jaHJvbm91cyBmaXJzdC1mYWlsdXJlL3dhbGwgbGF0Y2ggQkVGT1JFIGFueSBuZXcgYXdhaXQvb3V0cHV0LgogIH0KfQpmdW5jdGlvbiBvYnNlcnZlQ29udHJvbGxlcihyLHJlY2VpdmVkV2FsbCkgewogIGNvbnN0IG9ic2VydmF0aW9uPXtyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbCwKICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgIGhlbHBlcl9jb21wbGV0ZWQ6ZmFsc2UsaGVscGVyX2V4aXQ6bnVsbCxmaXh0dXJlX2ZpbmlzaGVkOmZhbHNlfTsKICAvLyBDb21wbGV0ZSBudW1lcmljIHJlc3VsdCBwcm9qZWN0aW9uIGlzIHJldGFpbmVkIEJFRk9SRSBjb21wYXJpc29ucy4KICByZWNvcmQuY29udHJvbGxlcl9vYnNlcnZhdGlvbnMucHVzaChvYnNlcnZhdGlvbik7cmV0YWluKCk7CiAgaWYodHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciIpIHsKICAgIGNvbnRyb2xsZXJKb2luZWQ9dHJ1ZTtyZWNvcmQuY29udHJvbGxlcl9leGl0PXIuZXhpdF9jb2RlO3JldGFpbigpOwogIH0KICBjb250cm9sbGVyQnVmZmVyKz10eXBlb2Ygci5vdXRwdXQ9PT0ic3RyaW5nIj9yLm91dHB1dDoiIjsKICBpZihjb250cm9sbGVyQnVmZmVyLmxlbmd0aD4xNjM4NCkgewogICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250cm9sbGVyQnVmZmVyPSIiO3JldHVybjsKICB9CiAgbGV0IGN1dDsKICB3aGlsZSgoY3V0PWNvbnRyb2xsZXJCdWZmZXIuaW5kZXhPZigiXG4iKSk+PTApIHsKICAgIGNvbnN0IGxpbmU9Y29udHJvbGxlckJ1ZmZlci5zbGljZSgwLGN1dCk7Y29udHJvbGxlckJ1ZmZlcj1jb250cm9sbGVyQnVmZmVyLnNsaWNlKGN1dCsxKTsKICAgIGlmKCFsaW5lLnRyaW0oKSljb250aW51ZTsKICAgIGxldCBldmVudDsKICAgIHRyeXtldmVudD1KU09OLnBhcnNlKGxpbmUpO31jYXRjaHsKICAgICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIscmVjZWl2ZWRXYWxsKTtjb250aW51ZTsKICAgIH0KICAgIGlmKE9iamVjdC5oYXNPd24oZXZlbnQsInVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiIpKQogICAgICBkaWFnbm9zdGljKGV2ZW50LnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbixldmVudC51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoZXZlbnQuZml4dHVyZV9yZWFkeT09PXRydWUpIHsKICAgICAgaWYoZXZlbnQuZ3VhcmRfcGlkIT09b3duLmd1YXJkX3BpZHx8ZXZlbnQuc2VydmVyX3BpZCE9PW93bi5zZXJ2ZXJfcGlkfHwKICAgICAgICAgZXZlbnQubGFiIT09b3duLmxhYnx8IU51bWJlci5pc1NhZmVJbnRlZ2VyKGV2ZW50LmhlbHBlcl9waWQpfHxldmVudC5oZWxwZXJfcGlkPD0wfHwKICAgICAgICAgW293bi5ndWFyZF9waWQsb3duLnNlcnZlcl9waWQsb3duLmJyb3dzZXJfcGlkXS5pbmNsdWRlcyhldmVudC5oZWxwZXJfcGlkKXx8CiAgICAgICAgIChvd24uaGVscGVyX3BpZCE9PW51bGwmJm93bi5oZWxwZXJfcGlkIT09ZXZlbnQuaGVscGVyX3BpZCkpCiAgICAgICAgbGF0Y2goImZpeHR1cmVfcmVhZHlfdW5jb25maXJtZWQiLHJlY2VpdmVkV2FsbCk7CiAgICAgIGVsc2UgewogICAgICAgIG93bi5oZWxwZXJfcGlkPWV2ZW50LmhlbHBlcl9waWQ7b3duLmZpeHR1cmVfcmVhZHk9dHJ1ZTtvd24ubGlzdGVuZXJfcGlkX3Byb29mPXRydWU7CiAgICAgICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICAgICAgfQogICAgfQogICAgaWYoZXZlbnQuaGVscGVyX2NvbXBsZXRlZD09PXRydWUpIHsKICAgICAgb2JzZXJ2YXRpb24uaGVscGVyX2NvbXBsZXRlZD10cnVlOwogICAgICBvYnNlcnZhdGlvbi5oZWxwZXJfZXhpdD10eXBlb2YgZXZlbnQuZXhpdD09PSJudW1iZXIiP2V2ZW50LmV4aXQ6bnVsbDsKICAgICAgcmV0YWluKCk7CiAgICAgIGlmKGV2ZW50LmV4aXQhPT0wKWxhdGNoKCJoZWxwZXJfZmFpbGVkIixyZWNlaXZlZFdhbGwpOwogICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSkKICAgICAgICBsYXRjaCgiaGVscGVyX2NvbXBsZXRlZF9iZWZvcmVfYXBwX2NoZWNrcG9pbnQiLHJlY2VpdmVkV2FsbCk7CiAgICB9CiAgICBpZihldmVudC5maXh0dXJlX2ZpbmlzaGVkPT09dHJ1ZSkgewogICAgICBvYnNlcnZhdGlvbi5maXh0dXJlX2ZpbmlzaGVkPXRydWU7cmV0YWluKCk7CiAgICAgIGlmKCFjbGVhbnVwU3RhcnRlZClsYXRjaCgiY29udHJvbGxlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50IixyZWNlaXZlZFdhbGwpOwogICAgfQogIH0KICBpZihjb250cm9sbGVySm9pbmVkJiYhY2xlYW51cFN0YXJ0ZWQpCiAgICBsYXRjaCgiY29udHJvbGxlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50IixyZWNlaXZlZFdhbGwpOwp9CmFzeW5jIGZ1bmN0aW9uIHBvbGxDb250cm9sbGVyKCkgewogIGlmKGNvbnRyb2xsZXJKb2luZWR8fCFjb250cm9sbGVyUG9sbEF2YWlsYWJsZSlyZXR1cm47CiAgdHJ5IHsKICAgIGNvbnN0IHI9YXdhaXQgdG9vbHMud3JpdGVfc3RkaW4oe3Nlc3Npb25faWQ6b3duLmV4ZWNfc2Vzc2lvbiwKICAgICAgY2hhcnM6IiIseWllbGRfdGltZV9tczo1MDAwLG1heF9vdXRwdXRfdG9rZW5zOjIwMDB9KTsKICAgIGNvbnN0IHJlY2VpdmVkV2FsbD1EYXRlLm5vdygpOwogICAgb2JzZXJ2ZUNvbnRyb2xsZXIocixyZWNlaXZlZFdhbGwpOwogIH0gY2F0Y2ggewogICAgY29udHJvbGxlclBvbGxBdmFpbGFibGU9ZmFsc2U7CiAgICBsYXRjaCgiY29udHJvbGxlcl9vYnNlcnZhdGlvbl91bmF2YWlsYWJsZSIsRGF0ZS5ub3coKSk7CiAgICBpZihjbGVhbnVwU3RhcnRlZCljbGVhbnVwRXJyb3IoImNvbnRyb2xsZXJfam9pbl91bmNvbmZpcm1lZCIpOwogIH0KfQphc3luYyBmdW5jdGlvbiB0aW1lZERyaXZlcihsYWJlbCxhbGxvY2F0aW9uLG9wZXJhdGlvbixwcm9qZWN0KSB7CiAgbGV0IHN0YXJ0PW51bGwsZW5kPW51bGwscmVzdWx0PW51bGwsc3RhdGU9InVua25vd24iOwogIHRyeXtzdGFydD1hd2FpdCBnZXRDbG9jaygpO31jYXRjaHtjbGVhbnVwRXJyb3IoImNsb2NrX3VuYXZhaWxhYmxlIik7fQogIGNvbnN0IGFjdGlvbj17bGFiZWwsc3RhcnRfY2xvY2s6c3RhcnQsZW5kX2Nsb2NrOm51bGwsYWxsb3dlZF9tczphbGxvY2F0aW9uLAogICAgcmVzdWx0X3N0YXRlOiJ1bmtub3duIixlbGFwc2VkX21zOm51bGwsb3Zlcl9idWRnZXQ6bnVsbH07CiAgcmVjb3JkLmFjdGlvbnMucHVzaChhY3Rpb24pO3JldGFpbigpOyAvLyBTdGFydCByZXRhaW5lZCBCRUZPUkUgZW50ZXJpbmcgRHJpdmVyIGNhbGwuCiAgdHJ5IHsKICAgIHJlc3VsdD1hd2FpdCBvcGVyYXRpb24oKTsKICAgIHN0YXRlPXJlc3VsdD8uaXNFcnJvcj09PXRydWU/InJlZnVzZWQiOiJyZXR1cm5lZCI7CiAgfSBjYXRjaCB7c3RhdGU9ImV4Y2VwdGlvbiI7fQogIHRyeXtlbmQ9YXdhaXQgZ2V0Q2xvY2soKTt9Y2F0Y2h7Y2xlYW51cEVycm9yKCJjbG9ja191bmF2YWlsYWJsZSIpO30KICBhY3Rpb24uZW5kX2Nsb2NrPWVuZDthY3Rpb24ucmVzdWx0X3N0YXRlPXN0YXRlOwogIHJldGFpbigpOyAvLyBFbmQvcmVzdWx0IHJldGFpbmVkIEJFRk9SRSBidWRnZXQgY29tcGFyaXNvbi4KICBpZihzdGFydCYmZW5kKSB7CiAgICBjb25zdCBkPW5zKGVuZCktbnMoc3RhcnQpOwogICAgaWYoZD49MG4pIHsKICAgICAgYWN0aW9uLmVsYXBzZWRfbXM9TnVtYmVyKGQvMTAwMDAwMG4pOwogICAgICBhY3Rpb24ub3Zlcl9idWRnZXQ9YWN0aW9uLmVsYXBzZWRfbXM+YWxsb2NhdGlvbjsKICAgIH0gZWxzZSBjbGVhbnVwRXJyb3IoImNsb2NrX2ludmFsaWQiKTsKICB9CiAgaWYoYWN0aW9uLm92ZXJfYnVkZ2V0KWNsZWFudXBFcnJvcigiZHJpdmVyX29wZXJhdGlvbl9vdmVyX2J1ZGdldCIpOwogIGlmKHN0YXRlIT09InJldHVybmVkIiljbGVhbnVwRXJyb3IoImRyaXZlcl9vcGVyYXRpb25fdW5jb25maXJtZWQiKTsKICBpZihwcm9qZWN0KXByb2plY3QocmVzdWx0LHN0YXRlKTsKICByZXRhaW4oKTsKfQphc3luYyBmdW5jdGlvbiBjbGVhbnVwKCkgewogIGlmKGNsZWFudXBTdGFydGVkKXJldHVybjsKICBjbGVhbnVwU3RhcnRlZD10cnVlO3JlY29yZC5jbGVhbnVwX3N0YXJ0ZWQ9dHJ1ZTtyZXRhaW4oKTsKICB0cnkgewogICAgY29uc3Qgcj1hd2FpdCBsb2NhbChTVE9QLHsKICAgICAgbGFiOm93bi5sYWIsZXZlbnRfb3V0Om93bi5ldmVudF9vdXQsCiAgICAgIGZpcnN0X2ZhaWx1cmU6cmVjb3JkLmZpcnN0X2ZhaWx1cmUsCiAgICAgIGZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXM6cmVjb3JkLmZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXMKICAgIH0pOwogICAgcmVjb3JkLmZpcnN0X2Nsb2NrPXIuY2xvY2s7cmVjb3JkLnN0b3Bfc3RhdGU9ci5tYXJrZXJfc3RhdGU7cmV0YWluKCk7CiAgICBpZihyLmV2ZW50X3N0YXRlIT09IndyaXR0ZW4iKWNsZWFudXBFcnJvcigib2JzZXJ2YXRpb25fbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICAgIGlmKHIubWFya2VyX3N0YXRlPT09InN0b3BfdW5jb25maXJtZWQiKWNsZWFudXBFcnJvcigic3RvcF91bmNvbmZpcm1lZCIpOwogICAgaWYoci5tYXJrZXJfc3RhdGU9PT0ib3duZXJzaGlwX3Vua25vd24iKWNsZWFudXBFcnJvcigib3duZXJzaGlwX3Vua25vd24iKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoInN0b3Bfb3JfY2xvY2tfcmVjb3JkX3VuYXZhaWxhYmxlIik7fQogIC8vIE5vIG91dHN0YW5kaW5nIERyaXZlciBjYWxsIGV4aXN0cyBoZXJlOiBhbGwgcGFnZSBjYWxscyBhYm92ZSB3ZXJlIGF3YWl0ZWQuCiAgLy8gT3JpZ2luYWwgc3RvcCBwcm90b2NvbCBhbHJlYWR5IGNhdXNlcyB0aGUgY29udHJvbGxlcidzIG93bmVkLWNoaWxkIGZpbmFsbHkuCiAgYXdhaXQgdGltZWREcml2ZXIoImtpbGxfYXBwIiwzMDAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2tpbGxfYXBwKHtwaWQ6b3duLmJyb3dzZXJfcGlkfSkpOwogIGF3YWl0IHRpbWVkRHJpdmVyKCJlbmRfc2Vzc2lvbiIsMTUwMDAsCiAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19lbmRfc2Vzc2lvbih7c2Vzc2lvbjpvd24uc2Vzc2lvbn0pLChyLHN0YXRlKT0+ewogICAgICByZWNvcmQuc2Vzc2lvbl9lbmRlZD1zdGF0ZT09PSJyZXR1cm5lZCImJgogICAgICAgIHI/LnN0cnVjdHVyZWRDb250ZW50Py5hY3RpdmU9PT1mYWxzZSYmci5zdHJ1Y3R1cmVkQ29udGVudC5zZXNzaW9uPT09b3duLnNlc3Npb247CiAgICAgIHJldGFpbigpOwogICAgfSk7CiAgYXdhaXQgdGltZWREcml2ZXIoImxpc3Rfd2luZG93cyIsNTAwMCwKICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2xpc3Rfd2luZG93cyh7cGlkOm93bi5icm93c2VyX3BpZH0pLChyLHN0YXRlKT0+ewogICAgICBjb25zdCB3aW5kb3dzPXI/LnN0cnVjdHVyZWRDb250ZW50Py53aW5kb3dzOwogICAgICByZWNvcmQud2luZG93X2NvdW50PXN0YXRlPT09InJldHVybmVkIiYmQXJyYXkuaXNBcnJheSh3aW5kb3dzKT93aW5kb3dzLmxlbmd0aDpudWxsOwogICAgICByZXRhaW4oKTsKICAgIH0pOwogIGxldCByZWFkU3RhcnQ9bnVsbDsKICB0cnl7cmVhZFN0YXJ0PWF3YWl0IGdldENsb2NrKCk7fWNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgY29uc3QgYWN0aW9uPXtsYWJlbDoib3duZWRfam9pbl9yZWFkYmFjayIsc3RhcnRfY2xvY2s6cmVhZFN0YXJ0LGVuZF9jbG9jazpudWxsLAogICAgYWxsb3dlZF9tczpudWxsLGVsYXBzZWRfbXM6bnVsbCxvdmVyX2J1ZGdldDpudWxsLHJlc3VsdF9zdGF0ZToidW5rbm93biJ9OwogIHJlY29yZC5hY3Rpb25zLnB1c2goYWN0aW9uKTtyZXRhaW4oKTsKICBpZihyZWFkU3RhcnQmJnJlY29yZC5maXJzdF9jbG9jaykgewogICAgYWN0aW9uLmFsbG93ZWRfbXM9TWF0aC5tYXgoMCw2MDAwMC1OdW1iZXIoKG5zKHJlYWRTdGFydCktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pKTsKICAgIHJldGFpbigpOwogIH0KICAvLyBDb250aW51ZSBlc3NlbnRpYWwgam9pbiBhZnRlcjYwcyBpZiBsYXRlOyBuZXZlciBjbGFpbSB0aGF0IGRlYWRsaW5lIGVuZm9yY2VkLgogIC8vIERvIG5vdCBwb2xsIGFub3RoZXIgZXhlYyBzZXNzaW9uIG9yIHNlbmQgYSBtYW51YWwgcHJvY2VzcyBzaWduYWwuCiAgd2hpbGUoIWNvbnRyb2xsZXJKb2luZWQmJmNvbnRyb2xsZXJQb2xsQXZhaWxhYmxlJiZEYXRlLm5vdygpPG93bi5zdGFydF9lcG9jaF9tcys5MDAwMDApIHsKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spIHsKICAgICAgdHJ5IHsKICAgICAgICBjb25zdCBjPWF3YWl0IGdldENsb2NrKCk7cmVjb3JkLmxhc3Rfam9pbl9jbG9jaz1jO3JldGFpbigpOwogICAgICAgIGlmKG5zKGMpLW5zKHJlY29yZC5maXJzdF9jbG9jayk+NjAwMDAwMDAwMDBuKQogICAgICAgICAgY2xlYW51cEVycm9yKCJjbGVhbnVwX2J1ZGdldF9leGNlZWRlZCIpOwogICAgICB9IGNhdGNoe2NsZWFudXBFcnJvcigiY2xvY2tfdW5hdmFpbGFibGUiKTt9CiAgICB9CiAgfQogIGlmKCFjb250cm9sbGVySm9pbmVkKWNsZWFudXBFcnJvcigiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIik7CiAgdHJ5IHsKICAgIGNvbnN0IHI9YXdhaXQgbG9jYWwoUkVBREJBQ0ssewogICAgICBvd25lZF9waWRzOltvd24uZ3VhcmRfcGlkLG93bi5zZXJ2ZXJfcGlkLG93bi5icm93c2VyX3BpZCwKICAgICAgICAuLi4ob3duLmhlbHBlcl9waWQ9PT1udWxsP1tdOltvd24uaGVscGVyX3BpZF0pXSwKICAgICAgbGFiOm93bi5sYWIsb3V0ZXJfb3V0Om93bi5vdXRlcl9vdXQsaGVscGVyX3BpZDpvd24uaGVscGVyX3BpZCxzZXJ2ZXJfcGlkOm93bi5zZXJ2ZXJfcGlkCiAgICB9KTsKICAgIGFjdGlvbi5lbmRfY2xvY2s9ci5jbG9jazthY3Rpb24ucmVzdWx0X3N0YXRlPSJyZXR1cm5lZCI7CiAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHM9ci5vd25lZF9jaGlsZF9leGl0czsKICAgIGRpYWdub3N0aWMoci51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sci51bmV4cGVjdGVkX29ic2VydmF0aW9uX3Byb2plY3Rpb24pOwogICAgaWYoTnVtYmVyLmlzU2FmZUludGVnZXIoci5oZWxwZXJfcGlkKSYmci5oZWxwZXJfcGlkPjApb3duLmhlbHBlcl9waWQ9ci5oZWxwZXJfcGlkOwogICAgcmVjb3JkLmZpbmFsX2Fic2VuY2U9e293bmVkX3BpZHM6ci5vd25lZF9waWRzLHBvcnRzOnIucG9ydHMsCiAgICAgIGxhYjpyLmxhYl9hYnNlbnQsd2luZG93X2NvdW50OnJlY29yZC53aW5kb3dfY291bnQ/P251bGwsCiAgICAgIHNlc3Npb25fZW5kZWQ6cmVjb3JkLnNlc3Npb25fZW5kZWQ9PT10cnVlfTsKICAgIHJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl90b19maW5hbF93YWxsX21zPXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPT09bnVsbD9udWxsOgogICAgICBEYXRlLm5vdygpLXJlY29yZC5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zOwogICAgcmV0YWluKCk7IC8vIEZ1bGwgZml4ZWQgcmVhZGJhY2svZXhpdHMvY2xvY2sgQkVGT1JFIGNvbXBhcmlzb25zLgogICAgaWYocmVhZFN0YXJ0KSB7CiAgICAgIGFjdGlvbi5lbGFwc2VkX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVhZFN0YXJ0KSkvMTAwMDAwMG4pOwogICAgICBhY3Rpb24ub3Zlcl9idWRnZXQ9YWN0aW9uLmFsbG93ZWRfbXM9PT1udWxsP251bGw6YWN0aW9uLmVsYXBzZWRfbXM+YWN0aW9uLmFsbG93ZWRfbXM7CiAgICB9CiAgICBpZihyZWNvcmQuZmlyc3RfY2xvY2spCiAgICAgIHJlY29yZC5sYXRjaF90b19hYnNlbmNlX21zPU51bWJlcigobnMoci5jbG9jayktbnMocmVjb3JkLmZpcnN0X2Nsb2NrKSkvMTAwMDAwMG4pOwogICAgaWYoYWN0aW9uLm92ZXJfYnVkZ2V0KWNsZWFudXBFcnJvcigiY2xlYW51cF9idWRnZXRfZXhjZWVkZWQiKTsKICAgIGNvbnN0IGE9cmVjb3JkLmZpbmFsX2Fic2VuY2U7CiAgICBpZighY29udHJvbGxlckpvaW5lZHx8IUFycmF5LmlzQXJyYXkocmVjb3JkLm93bmVkX2NoaWxkX2V4aXRzKXx8CiAgICAgICByZWNvcmQub3duZWRfY2hpbGRfZXhpdHMubGVuZ3RoIT09KG93bi5oZWxwZXJfcGlkPT09bnVsbD82OjcpKQogICAgICBjbGVhbnVwRXJyb3IoImNoaWxkX3JlYXBfaW5jb21wbGV0ZSIpOwogICAgaWYoIU9iamVjdC52YWx1ZXMoYS5vd25lZF9waWRzKS5ldmVyeSh2PT52PT09dHJ1ZSl8fAogICAgICAgIU9iamVjdC52YWx1ZXMoYS5wb3J0cykuZXZlcnkodj0+dj09PXRydWUpfHxhLmxhYiE9PXRydWV8fAogICAgICAgYS53aW5kb3dfY291bnQhPT0wfHxhLnNlc3Npb25fZW5kZWQhPT10cnVlKQogICAgICBjbGVhbnVwRXJyb3IoIm93bmVkX3Jlc291cmNlX2Fic2VuY2VfdW5wcm92ZW4iKTsKICB9IGNhdGNoIHthY3Rpb24ucmVzdWx0X3N0YXRlPSJleGNlcHRpb24iO2NsZWFudXBFcnJvcigib3duZWRfcmVhZGJhY2tfdW5hdmFpbGFibGUiKTt9CiAgY2xlYW51cEVycm9yKCJmaXJzdF9ldmVudF91bnByb3ZlbiIpOwogIHJlY29yZC53aG9sZV9jbGVhbnVwX3dpdGhpbjYwX3Byb3Zlbj1mYWxzZTtyZXRhaW4oKTsKICAvLyBFeGNsdXNpdmUgbmV3IGZpbGUgb25seTsgZXhpc3Rpbmcgb3V0ZXIvaGVscGVyL3Byb3ZpZGVyIGV2aWRlbmNlIHVudG91Y2hlZC4KICB0cnkgewogICAgY29uc3QgY21kPSJweXRob24zIC1jICIrc3EoUEVSU0lTVCkrIiAiK3NxKG93bi5jbGVhbnVwX291dCkrIiAiK3NxKEpTT04uc3RyaW5naWZ5KHJlY29yZCkpOwogICAgY29uc3Qgcj1hd2FpdCB0b29scy5leGVjX2NvbW1hbmQoe2NtZCx3b3JrZGlyOm93bi53b3Jrc3BhY2UsCiAgICAgIHlpZWxkX3RpbWVfbXM6MTAwMDAsbWF4X291dHB1dF90b2tlbnM6NTAwfSk7CiAgICByZWNvcmQubG9jYWxfdG9vbF9yZWNlaXB0cy5wdXNoKHtsYWJlbDoicGVyc2lzdCIsCiAgICAgIGV4aXQ6dHlwZW9mIHIuZXhpdF9jb2RlPT09Im51bWJlciI/ci5leGl0X2NvZGU6bnVsbCwKICAgICAgc2Vzc2lvbl9pZDp0eXBlb2Ygci5zZXNzaW9uX2lkPT09Im51bWJlciI/ci5zZXNzaW9uX2lkOm51bGx9KTtyZXRhaW4oKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZHx8ci5leGl0X2NvZGUhPT0wKQogICAgICBjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTsKICB9IGNhdGNoIHtjbGVhbnVwRXJyb3IoImNsZWFudXBfbWV0YWRhdGFfd3JpdGVfdW5jb25maXJtZWQiKTt9CiAgY29udHJvbGxlckJ1ZmZlcj0iIjtvd24uY2xvc2VkPXRydWU7c3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKfQphc3luYyBmdW5jdGlvbiBjaGVja2VkKGxhYmVsLG9wZXJhdGlvbixwcmVkaWNhdGUpIHsKICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZSE9PW51bGwpcmV0dXJuIG51bGw7CiAgaWYoRGF0ZS5ub3coKT49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkgewogICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuIG51bGw7CiAgfQogIGxldCByZXN1bHQscmVjZWl2ZWRXYWxsOwogIHRyeSB7CiAgICByZXN1bHQ9YXdhaXQgb3BlcmF0aW9uKCk7cmVjZWl2ZWRXYWxsPURhdGUubm93KCk7CiAgICByZWNvcmQub2JzZXJ2YXRpb25fcmVjZWlwdHMucHVzaCh7a2luZDpsYWJlbCxyZWNlaXZlZF93YWxsX21zOnJlY2VpdmVkV2FsbH0pO3JldGFpbigpOwogICAgaWYocmVzdWx0Py5pc0Vycm9yPT09dHJ1ZSlsYXRjaCgiYnJvd3Nlcl90b29sX3JlZnVzZWQiLHJlY2VpdmVkV2FsbCk7CiAgICBlbHNlIGlmKCFwcmVkaWNhdGUocmVzdWx0KSlsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIscmVjZWl2ZWRXYWxsKTsKICB9IGNhdGNoIHtsYXRjaCgiYnJvd3Nlcl90b29sX29yX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7fQogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVhbnVwKCk7CiAgcmV0dXJuIHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbD9yZXN1bHQ6bnVsbDsKfQpjb25zdCBzbmFwc2hvdEFyZ3M9e3Nlc3Npb246b3duLnNlc3Npb24sdGFyZ2V0X2lkOm93bi50YXJnZXRfaWQsdGFiX2lkOm93bi50YWJfaWQsCiAgc25hcHNob3RfZm9ybWF0OiJzZW1hbnRpY192MiIsaW5jbHVkZV9zY3JlZW5zaG90OmZhbHNlfTsKZnVuY3Rpb24gcGFnZShyZXN1bHQsa2luZCkgewogIGNvbnN0IHM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudCxub2Rlcz1BcnJheS5pc0FycmF5KHM/LmNvbnRlbnRfcmVmcyk/cy5jb250ZW50X3JlZnM6W107CiAgY29uc3QgZmxhZ3M9ewogICAgc3RhdHVzX29rOnJlc3VsdD8uaXNFcnJvciE9PXRydWUmJnM/LnN0YXR1cz09PSJvayIsCiAgICBjb21wbGV0ZTpzPy5zbmFwc2hvdD8uY29tcGxldGU9PT10cnVlLAogICAgaGVhZGluZ19tYXRjaDpub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09CiAgICAgIChraW5kPT09InByb3RlY3RlZF9hZnRlciI/IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiOiJMb2NhbCBkZW1vIikpLAogICAgZXJyb3JfbWF0Y2g6bm9kZXMuc29tZShuPT5uLm5hbWU9PT0iTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LiIpLAogICAgcmVxdWlyZWRfdGV4dDpub2Rlcy5zb21lKG49Pm4ubmFtZT09PShraW5kPT09InByb3RlY3RlZF9iZWZvcmUiPyJTaWduIGluIHJlcXVpcmVkLiI6CiAgICAgIGtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIj8iU2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS4iOgogICAgICAiVXNlIFNpZ24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLiIpKSwKICAgIGludGVyYWN0aXZlX3JlZl9wcmVzZW50OkFycmF5LmlzQXJyYXkocz8ucmVmcykmJgogICAgICBzLnJlZnMuc29tZShuPT5BcnJheS5pc0FycmF5KG4uYWN0aW9ucykmJm4uYWN0aW9ucy5pbmNsdWRlcygiY2xpY2siKSkKICB9OwogIHJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M9e2tpbmQsLi4uZmxhZ3N9O3JldGFpbigpOwogIC8vIE9ubHkgcHJvdmlkZXIgcmVmcyBhbmQgZml4ZWQgcHVibGljLWxhYmVsIG1hdGNoZXMgc3Vydml2ZSB0aGlzIHJhdyBzbmFwc2hvdC4KICBvd24uZnJlc2hfcmVmcz1BcnJheS5pc0FycmF5KHM/LnJlZnMpP3MucmVmcy5maWx0ZXIobj0+dHlwZW9mIG4ucmVmPT09InN0cmluZyImJgogICAgL15wWzAtOV0rOlswLTldKyQvLnRlc3Qobi5yZWYpKS5tYXAobj0+bi5yZWYpOltdOwogIHN0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7CiAgaWYoZmxhZ3MuZXJyb3JfbWF0Y2gpcmV0dXJuIGZhbHNlOwogIGlmKCFmbGFncy5zdGF0dXNfb2t8fCFmbGFncy5jb21wbGV0ZSlyZXR1cm4gZmFsc2U7CiAgaWYoa2luZD09PSJnZW5lcmljX2NoZWNrZWQiKSB7CiAgICBpZihub2Rlcy5zb21lKG49Pm4ucm9sZT09PSJoZWFkaW5nIiYmbi5uYW1lPT09IlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiKSYmCiAgICAgICBub2Rlcy5zb21lKG49Pm4ubmFtZT09PSJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZhaWxhYmxlLiIpKQogICAgICByZWNvcmQucHJvdGVjdGVkX2FmdGVyX3BhZ2U9dHJ1ZTsKICAgIHJldHVybiB0cnVlOwogIH0KICBjb25zdCBvaz1mbGFncy5oZWFkaW5nX21hdGNoJiZmbGFncy5yZXF1aXJlZF90ZXh0JiYKICAgIChraW5kIT09ImFwcGxpY2F0aW9uInx8ZmxhZ3MuaW50ZXJhY3RpdmVfcmVmX3ByZXNlbnQpOwogIGlmKG9rJiZraW5kPT09InByb3RlY3RlZF9hZnRlciIpcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlPXRydWU7CiAgcmV0dXJuIG9rOwp9CmFzeW5jIGZ1bmN0aW9uIG5hdmlnYXRlQW5kU25hcHNob3QodXJsLGtpbmQpIHsKICBpZihhd2FpdCBjaGVja2VkKCJicm93c2VyX25hdmlnYXRpb24iLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19icm93c2VyX25hdmlnYXRlKHtzZXNzaW9uOm93bi5zZXNzaW9uLAogICAgICAgIHRhcmdldF9pZDpvd24udGFyZ2V0X2lkLHRhYl9pZDpvd24udGFiX2lkLHVybH0pLAogICAgICByPT5yPy5pc0Vycm9yIT09dHJ1ZSkpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLHI9PnBhZ2UocixraW5kKSk7CiAgfQp9CmFzeW5jIGZ1bmN0aW9uIGVudHJ5KCkgewogIC8vIFRoZSBleGFjdCBEcml2ZXItb3duZWQgYmxhbmsgaGFuZGxlcyBhbHJlYWR5IGV4aXN0IHdoaWxlIHRoZTE4MHMgZ2F0ZSB3YWl0cy4KICAvLyBUaGVyZSBpcyBubyBtb2RlbCB5aWVsZCwgdG9vbC1kZXNjcmlwdGlvbiBsb2FkIG9yIHJlYmluZCBhZnRlciB0aGlzIG1hcmtlci4KICBjb25zdCBtYXJrZXJDb21tYW5kPSJweXRob24zIC1jICIrc3EoTUFSS0VSKSsiICIrc3Eob3duLmxhYikrIiAiKwogICAgc3Eob3duLmd1YXJkX3BpZCkrIiAiK3NxKG93bi5zZXJ2ZXJfcGlkKTsKICBhd2FpdCBjaGVja2VkKCJwcmVwYXJlZF9tYXJrZXIiLAogICAgKCk9PnRvb2xzLmV4ZWNfY29tbWFuZCh7Y21kOm1hcmtlckNvbW1hbmQsd29ya2Rpcjpvd24ud29ya3NwYWNlLAogICAgICB5aWVsZF90aW1lX21zOjEwMDAwLG1heF9vdXRwdXRfdG9rZW5zOjUwMH0pLHI9PnsKICAgICAgcmVjb3JkLmxvY2FsX3Rvb2xfcmVjZWlwdHMucHVzaCh7bGFiZWw6Im1hcmtlciIsCiAgICAgICAgZXhpdDp0eXBlb2Ygci5leGl0X2NvZGU9PT0ibnVtYmVyIj9yLmV4aXRfY29kZTpudWxsLAogICAgICAgIHNlc3Npb25faWQ6dHlwZW9mIHIuc2Vzc2lvbl9pZD09PSJudW1iZXIiP3Iuc2Vzc2lvbl9pZDpudWxsfSk7cmV0YWluKCk7CiAgICAgIGlmKHIuc2Vzc2lvbl9pZCE9PXVuZGVmaW5lZCljbGVhbnVwRXJyb3IoIm93bmVkX2xvY2FsX2NvbW1hbmRfdW5qb2luZWQiKTsKICAgICAgcmV0dXJuIHIuZXhpdF9jb2RlPT09MCYmci5zZXNzaW9uX2lkPT09dW5kZWZpbmVkJiYKICAgICAgICBKU09OLnBhcnNlKHIub3V0cHV0KS5wcmVwYXJlZF9tYXJrZXJfd3JpdHRlbj09PXRydWU7CiAgICB9KTsKICBjb25zdCByZWFkeURlYWRsaW5lPURhdGUubm93KCkrMzAwMDA7CiAgd2hpbGUocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUmJkRhdGUubm93KCk8cmVhZHlEZWFkbGluZSkKICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsJiZvd24uZml4dHVyZV9yZWFkeSE9PXRydWUpCiAgICBsYXRjaCgiZml4dHVyZV9yZWFkeV91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOyAvLyBPTkUgaW1tZWRpYXRlIHByZS1uYXZpZ2F0aW9uIHBvbGwsIG5vIFJQIEhUVFAgcHJvYmUuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKXthd2FpdCBjbGVhbnVwKCk7cmV0dXJuO30KICBhd2FpdCBuYXZpZ2F0ZUFuZFNuYXBzaG90KCJodHRwOi8vbG9jYWxob3N0OjMwMDAvcHJvdGVjdGVkIiwicHJvdGVjdGVkX2JlZm9yZSIpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbClhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgYXdhaXQgbmF2aWdhdGVBbmRTbmFwc2hvdCgiaHR0cDovL2xvY2FsaG9zdDozMDAwLyIsImFwcGxpY2F0aW9uIik7CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKWF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIE9ORSBwb3N0LWVudHJ5IHBvbGwuCiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CiAgICBvd24ucGhhc2U9ImJyb3dzZXJfZGVjaXNpb24iOwogICAgc3RvcmUoImQwMV9pbW1lZGlhdGVfb3duZWRfaGFuZGxlcyIsb3duKTsKICB9CiAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKfQphc3luYyBmdW5jdGlvbiBjb250aW51YXRpb24oKSB7CiAgY29uc3Qgc3BlYz1vd24ubmV4dF9kZWNpc2lvbjsKICBjb25zdCBhbGxvd2VkS2luZHM9WyJjbGljayIsInVzZXJuYW1lIiwicGFzc3dvcmQiLCJzbmFwc2hvdCIsInByb3RlY3RlZF9hZnRlciJdOwogIGlmKCFzcGVjfHwhYWxsb3dlZEtpbmRzLmluY2x1ZGVzKHNwZWMua2luZCkpIHsKICAgIGxhdGNoKCJicm93c2VyX2RlY2lzaW9uX3VuY29uZmlybWVkIixEYXRlLm5vdygpKTthd2FpdCBjbGVhbnVwKCk7cmV0dXJuOwogIH0KICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCl7YXdhaXQgY2xlYW51cCgpO3JldHVybjt9CiAgaWYoc3BlYy5raW5kPT09InByb3RlY3RlZF9hZnRlciIpIHsKICAgIC8vIFJlYWQgdGhlIGZyZXNoIGNhbGxiYWNrLWZvbGxvd2luZyBwcm90ZWN0ZWQgcGFnZTsgZG8gbm90IHNlbmQgYW5vdGhlciBSUCByZXF1ZXN0LgogICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2dldF9icm93c2VyX3N0YXRlKHNuYXBzaG90QXJncykscj0+cGFnZShyLCJwcm90ZWN0ZWRfYWZ0ZXIiKSk7CiAgfSBlbHNlIGlmKHNwZWMua2luZD09PSJzbmFwc2hvdCIpIHsKICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfc25hcHNob3QiLAogICAgICAoKT0+dG9vbHMubWNwX19jdWFfZHJpdmVyX19nZXRfYnJvd3Nlcl9zdGF0ZShzbmFwc2hvdEFyZ3MpLAogICAgICByPT5wYWdlKHIsImdlbmVyaWNfY2hlY2tlZCIpJiZwdWJsaWNQcmVkaWNhdGUocixzcGVjKSk7CiAgfSBlbHNlIHsKICAgIGlmKHR5cGVvZiBzcGVjLnJlZiE9PSJzdHJpbmcifHwhQXJyYXkuaXNBcnJheShvd24uZnJlc2hfcmVmcyl8fAogICAgICAgIW93bi5mcmVzaF9yZWZzLmluY2x1ZGVzKHNwZWMucmVmKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIG93bi5mcmVzaF9yZWZzPVtdO3N0b3JlKCJkMDFfaW1tZWRpYXRlX293bmVkX2hhbmRsZXMiLG93bik7IC8vIFNpbmdsZS11c2Ugc25hcHNob3QgcmVmLgogICAgY29uc3QgYXJncz17c2Vzc2lvbjpvd24uc2Vzc2lvbix0YXJnZXRfaWQ6b3duLnRhcmdldF9pZCx0YWJfaWQ6b3duLnRhYl9pZCxyZWY6c3BlYy5yZWZ9OwogICAgY29uc3Qgb3BlcmF0aW9uPXNwZWMua2luZD09PSJjbGljayI/CiAgICAgICgpPT50b29scy5tY3BfX2N1YV9kcml2ZXJfX2Jyb3dzZXJfY2xpY2soey4uLmFyZ3MsaW5wdXRfcm91dGU6ImRvbV9ldmVudCJ9KToKICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fYnJvd3Nlcl90eXBlKHsuLi5hcmdzLHJlcGxhY2U6dHJ1ZSwKICAgICAgICB0ZXh0OnNwZWMua2luZD09PSJ1c2VybmFtZSI/ImFkbWluIjpsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKX0pOwogICAgaWYoc3BlYy5raW5kPT09InBhc3N3b3JkIiYmKHR5cGVvZiBsb2FkKCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXQiKSE9PSJzdHJpbmcifHwKICAgICAgICEvXltBLVphLXowLTlfLV17MzAsMTI4fSQvLnRlc3QobG9hZCgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IikpKSkgewogICAgICBsYXRjaCgiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsRGF0ZS5ub3coKSk7YXdhaXQgY2xlYW51cCgpO3JldHVybjsKICAgIH0KICAgIGF3YWl0IGNoZWNrZWQoImJyb3dzZXJfaW5wdXQiLG9wZXJhdGlvbixyPT57CiAgICAgIGNvbnN0IHM9cj8uc3RydWN0dXJlZENvbnRlbnQ7CiAgICAgIHJldHVybiByPy5pc0Vycm9yIT09dHJ1ZSYmWyJjb25maXJtZWQiLCJ1bnZlcmlmaWFibGUiXS5pbmNsdWRlcyhzPy5lZmZlY3QpOwogICAgfSk7IC8vIERpc3BhdGNoIGFsb25lIG5ldmVyIGVhcm5zIGFuIGFwcGxpY2F0aW9uIG91dGNvbWUgb3Igam91cm5leSBjcmVkaXQuCiAgICBzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsKICAgIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbCkKICAgICAgYXdhaXQgY2hlY2tlZCgiYnJvd3Nlcl9zbmFwc2hvdCIsCiAgICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fZ2V0X2Jyb3dzZXJfc3RhdGUoc25hcHNob3RBcmdzKSwKICAgICAgICByPT5wYWdlKHIsImdlbmVyaWNfY2hlY2tlZCIpJiZwdWJsaWNQcmVkaWNhdGUocixzcGVjKSk7CiAgfQogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVsbClhd2FpdCBwb2xsQ29udHJvbGxlcigpOwogIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVhbnVwKCk7CiAgZWxzZSBpZihzcGVjLmtpbmQ9PT0icHJvdGVjdGVkX2FmdGVyIikgewogICAgcmVjb3JkLmNsZWFudXBfcmVxdWVzdGVkX3dhbGxfbXM9RGF0ZS5ub3coKTtyZXRhaW4oKTsKICAgIGF3YWl0IGNsZWFudXAoKTsKICB9Cn0KZnVuY3Rpb24gcHVibGljUHJlZGljYXRlKHJlc3VsdCxzcGVjKSB7CiAgY29uc3Qgbm9kZXM9cmVzdWx0Py5zdHJ1Y3R1cmVkQ29udGVudD8uY29udGVudF9yZWZzOwogIC8vIFJvb3QgbXVzdCBwaW4gb25lIHByaW50ZWQgcHVibGljIGV4cGVjdGVkIGxhYmVsL3JvbGUgYmVmb3JlIHRoYXQgYWN0aW9uLgogIC8vIE5vIHJhdyBmaWVsZCB2YWx1ZSwgVVJMLCBzdWJqZWN0LCBxdWVyeSBvciBhcmJpdHJhcnkgcmVnZXggcHJlZGljYXRlIGlzIGFsbG93ZWQuCiAgY29uc3Qgcm9sZXM9WyJoZWFkaW5nIiwiYnV0dG9uIiwic3RhdGljdGV4dCIsInRleHRib3giXTsKICBjb25zdCBsYWJlbHM9WyJVc2VybmFtZSIsIlBhc3N3b3JkIiwiQXV0aGVudGljYXRvciBvciByZWNvdmVyeSBjb2RlIiwiU2lnbiBpbiIsCiAgICAiTG9jYWwgZGVtbyIsIlByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MiLAogICAgIlNpZ25lZCBpbi4gUHJvdGVjdGVkIGFwcGxpY2F0aW9uIGFjY2VzcyBpcyBhdmFpbGFibGUuIl07CiAgcmV0dXJuIEFycmF5LmlzQXJyYXkobm9kZXMpJiZyb2xlcy5pbmNsdWRlcyhzcGVjLmV4cGVjdGVkX3JvbGUpJiYKICAgIGxhYmVscy5pbmNsdWRlcyhzcGVjLmV4cGVjdGVkX2xhYmVsKSYmCiAgICBub2Rlcy5zb21lKG49Pm4ucm9sZT09PXNwZWMuZXhwZWN0ZWRfcm9sZSYmbi5uYW1lPT09c3BlYy5leHBlY3RlZF9sYWJlbCk7Cn0KdHJ5IHsKICBpZihvd24ucGhhc2U9PT0icHJlcGFyZWRfZW50cnkiKWF3YWl0IGVudHJ5KCk7ZWxzZSBhd2FpdCBjb250aW51YXRpb24oKTsKfSBjYXRjaCB7CiAgbGF0Y2goImNvbXBvc2VkX2RlY2lzaW9uX2V4Y2VwdGlvbiIsRGF0ZS5ub3coKSk7CiAgYXdhaXQgY2xlYW51cCgpOwp9CmlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbCkgewogIHRleHQoe3Jlc3VsdDoiZmFpbGVkIixmaXJzdF9mYWlsdXJlOnJlY29yZC5maXJzdF9mYWlsdXJlLAogICAgdW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uOnJlY29yZC51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24sCiAgICBkaWFnbm9zdGljX2Vycm9yczpyZWNvcmQuZGlhZ25vc3RpY19lcnJvcnMsY2xlYW51cF9lcnJvcnM6cmVjb3JkLmNsZWFudXBfZXJyb3JzLAogICAgZmluYWxfYWJzZW5jZTpyZWNvcmQuZmluYWxfYWJzZW5jZSxjb250cm9sbGVyX2V4aXQ6cmVjb3JkLmNvbnRyb2xsZXJfZXhpdCwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlLAogICAgcmVzb3VyY2VfcmVsZWFzZV9wcm92ZW46cmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKX0pOwp9IGVsc2UgewogIHRleHQoe3Jlc3VsdDoiYnJvd3Nlcl9kZWNpc2lvbl9vYnNlcnZlZF9vbmx5IixwYWdlX2ZsYWdzOnJlY29yZC5sYXN0X3BhZ2VfZmxhZ3M/P251bGwsCiAgICBqb3VybmV5X2NyZWRpdDpmYWxzZSxjbGVhbnVwX2Vycm9yczpyZWNvcmQuY2xlYW51cF9lcnJvcnMsCiAgICByZXNvdXJjZV9yZWxlYXNlX3Byb3ZlbjpyZWNvcmQuY2xlYW51cF9zdGFydGVkPT09dHJ1ZSYmcmVjb3JkLmZpbmFsX2Fic2VuY2UhPT1udWxsJiYKICAgICAgIXJlY29yZC5jbGVhbnVwX2Vycm9ycy5zb21lKHY9PlsKICAgICAgICAiY2hpbGRfcmVhcF9pbmNvbXBsZXRlIiwib3duZWRfcmVzb3VyY2VfYWJzZW5jZV91bnByb3ZlbiIsCiAgICAgICAgIm93bmVkX3JlYWRiYWNrX3VuYXZhaWxhYmxlIiwib3duZWRfbG9jYWxfY29tbWFuZF91bmpvaW5lZCJdLmluY2x1ZGVzKHYpKSwKICAgIHdob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuOmZhbHNlfSk7Cn0K",
  "diff_b64": "LS0tIGltbXV0YWJsZS1jNWQ0ZDE3My1jZWxsLWQ3Mjc4NzgwCisrKyBERVNJR04tb25seS1GMi1GMy1GNApAQCAtMTM5LDcgKzEzOSw5IEBACiAgICAgICBvYnNlcnZhdGlvbi5oZWxwZXJfZXhpdD10eXBlb2YgZXZlbnQuZXhpdD09PSJudW1iZXIiP2V2ZW50LmV4aXQ6bnVsbDsKICAgICAgIHJldGFpbigpOwogICAgICAgaWYoZXZlbnQuZXhpdCE9PTApbGF0Y2goImhlbHBlcl9mYWlsZWQiLHJlY2VpdmVkV2FsbCk7Ci0gICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSkKKyAgICAgIGVsc2UgaWYoIWNsZWFudXBTdGFydGVkJiZyZWNvcmQucHJvdGVjdGVkX2FmdGVyX3BhZ2UhPT10cnVlJiYKKyAgICAgICAgICAgICAgIShvd24ucGhhc2U9PT0iYnJvd3Nlcl9kZWNpc2lvbiImJgorICAgICAgICAgICAgICAgIG93bi5uZXh0X2RlY2lzaW9uPy5raW5kPT09InByb3RlY3RlZF9hZnRlciIpKQogICAgICAgICBsYXRjaCgiaGVscGVyX2NvbXBsZXRlZF9iZWZvcmVfYXBwX2NoZWNrcG9pbnQiLHJlY2VpdmVkV2FsbCk7CiAgICAgfQogICAgIGlmKGV2ZW50LmZpeHR1cmVfZmluaXNoZWQ9PT10cnVlKSB7CkBAIC0xODksNiArMTkxLDcgQEAKICAgcmV0YWluKCk7CiB9CiBhc3luYyBmdW5jdGlvbiBjbGVhbnVwKCkgeworICBzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsgLy8gQ2xlYXIgYmVmb3JlIGFueSBjbGVhbnVwIGF3YWl0LgogICBpZihjbGVhbnVwU3RhcnRlZClyZXR1cm47CiAgIGNsZWFudXBTdGFydGVkPXRydWU7cmVjb3JkLmNsZWFudXBfc3RhcnRlZD10cnVlO3JldGFpbigpOwogICB0cnkgewpAQCAtNDE1LDcgKzQxOCw3IEBACiAgICAgICBjb25zdCBzPXI/LnN0cnVjdHVyZWRDb250ZW50OwogICAgICAgcmV0dXJuIHI/LmlzRXJyb3IhPT10cnVlJiZbImNvbmZpcm1lZCIsInVudmVyaWZpYWJsZSJdLmluY2x1ZGVzKHM/LmVmZmVjdCk7CiAgICAgfSk7IC8vIERpc3BhdGNoIGFsb25lIG5ldmVyIGVhcm5zIGFuIGFwcGxpY2F0aW9uIG91dGNvbWUgb3Igam91cm5leSBjcmVkaXQuCi0gICAgc3RvcmUoImQwMV9mcmVzaF9wYXNzd29yZF9pbnB1dCIsbnVsbCk7CisgICAgaWYoc3BlYy5raW5kPT09InBhc3N3b3JkIilzdG9yZSgiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0IixudWxsKTsKICAgICBpZihyZWNvcmQuZmlyc3RfZmFpbHVyZT09PW51bGwpCiAgICAgICBhd2FpdCBjaGVja2VkKCJicm93c2VyX3NuYXBzaG90IiwKICAgICAgICAgKCk9PnRvb2xzLm1jcF9fY3VhX2RyaXZlcl9fZ2V0X2Jyb3dzZXJfc3RhdGUoc25hcHNob3RBcmdzKSwKQEAgLTQ0Miw2ICs0NDUsMjEgQEAKIH0KIHRyeSB7CiAgIGlmKG93bi5waGFzZT09PSJwcmVwYXJlZF9lbnRyeSIpYXdhaXQgZW50cnkoKTtlbHNlIGF3YWl0IGNvbnRpbnVhdGlvbigpOworICAvLyBEbyBub3QgY2FycnkgdW52YWxpZGF0ZWQgcGFydGlhbCBjb250cm9sbGVyIHRleHQgYWNyb3NzIHRoaXMgY2VsbCBib3VuZGFyeS4KKyAgd2hpbGUoY29udHJvbGxlckJ1ZmZlci5sZW5ndGg+MCYmcmVjb3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSB7CisgICAgY29uc3QgYmVmb3JlUG9sbFdhbGw9RGF0ZS5ub3coKTsKKyAgICBpZihiZWZvcmVQb2xsV2FsbD49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkgeworICAgICAgbGF0Y2goImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIixiZWZvcmVQb2xsV2FsbCk7YnJlYWs7CisgICAgfQorICAgIGlmKGNvbnRyb2xsZXJKb2luZWR8fCFjb250cm9sbGVyUG9sbEF2YWlsYWJsZSkgeworICAgICAgbGF0Y2goImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25faW52YWxpZCIsYmVmb3JlUG9sbFdhbGwpO2JyZWFrOworICAgIH0KKyAgICBhd2FpdCBwb2xsQ29udHJvbGxlcigpOyAvLyBTYW1lIG93bmVkIHNlc3Npb247IG9ic2VydmVyIHJldGFpbnMgcmVjZWlwdCBmaXJzdC4KKyAgICBjb25zdCBhZnRlclBvbGxXYWxsPURhdGUubm93KCk7CisgICAgaWYoYWZ0ZXJQb2xsV2FsbD49b3duLnN0YXJ0X2Vwb2NoX21zKzg0MDAwMCkKKyAgICAgIGxhdGNoKCJicm93c2VyX2FjdGl2ZV9kZWFkbGluZSIsYWZ0ZXJQb2xsV2FsbCk7CisgIH0KKyAgaWYocmVjb3JkLmZpcnN0X2ZhaWx1cmUhPT1udWxsKWF3YWl0IGNsZWFudXAoKTsKIH0gY2F0Y2ggewogICBsYXRjaCgiY29tcG9zZWRfZGVjaXNpb25fZXhjZXB0aW9uIixEYXRlLm5vdygpKTsKICAgYXdhaXQgY2xlYW51cCgpOwo=",
  "candidate_sha256": "7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8",
  "baseline_sha256": "d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d",
  "diff_sha256": "395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144",
  "logic_sha256": "639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054",
  "edits": [
    {
      "id": "F2-cleanup",
      "old": "  if(cleanupStarted)return;\n  cleanupStarted=true;record.cleanup_started=true;retain();",
      "next": "  store(\"d01_fresh_password_input\",null); // Clear before any cleanup await.\n  if(cleanupStarted)return;\n  cleanupStarted=true;record.cleanup_started=true;retain();"
    },
    {
      "id": "F2-password",
      "old": "    store(\"d01_fresh_password_input\",null);\n    if(record.first_failure===null)",
      "next": "    if(spec.kind===\"password\")store(\"d01_fresh_password_input\",null);\n    if(record.first_failure===null)"
    },
    {
      "id": "F3-final-snapshot",
      "old": "      else if(!cleanupStarted&&record.protected_after_page!==true)\n        latch(\"helper_completed_before_app_checkpoint\",receivedWall);",
      "next": "      else if(!cleanupStarted&&record.protected_after_page!==true&&\n              !(own.phase===\"browser_decision\"&&\n                own.next_decision?.kind===\"protected_after\"))\n        latch(\"helper_completed_before_app_checkpoint\",receivedWall);"
    },
    {
      "id": "F4-drain",
      "old": "  if(own.phase===\"prepared_entry\")await entry();else await continuation();\n} catch {",
      "next": "  if(own.phase===\"prepared_entry\")await entry();else await continuation();\n  // Do not carry unvalidated partial controller text across this cell boundary.\n  while(controllerBuffer.length>0&&record.first_failure===null) {\n    const beforePollWall=Date.now();\n    if(beforePollWall>=own.start_epoch_ms+840000) {\n      latch(\"browser_active_deadline\",beforePollWall);break;\n    }\n    if(controllerJoined||!controllerPollAvailable) {\n      latch(\"controller_observation_invalid\",beforePollWall);break;\n    }\n    await pollController(); // Same owned session; observer retains receipt first.\n    const afterPollWall=Date.now();\n    if(afterPollWall>=own.start_epoch_ms+840000)\n      latch(\"browser_active_deadline\",afterPollWall);\n  }\n  if(record.first_failure!==null)await cleanup();\n} catch {"
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
    "metadata_cap"
  ]
};

const LIMIT_MS=30000, MAX_CELLS=75, MAX_CALLS=2500, MAX_TRACE=512;
const PASSWORD="D01_SYNTHETIC_PASSWORD_SENTINEL_0123456789ABCDEFG";
const PARTIAL="D01_SYNTHETIC_PARTIAL_SENTINEL_PRIVATE";
const EXCEPTION="D01_SYNTHETIC_EXCEPTION_SENTINEL_PRIVATE";
const PRIVATE=[PASSWORD,PARTIAL,EXCEPTION];
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
  ensure(ds[0]==="--- immutable-c5d4d173-cell-d7278780\n"&&
    ds[1]==="+++ DESIGN-only-F2-F3-F4\n","diff_headers");
  while(i<ds.length){
    const m=/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@\n$/.exec(ds[i++]);
    ensure(m!==null,"diff_hunk");hunks++;const start=Number(m[3])-1;
    ensure(cursor<=start,"diff_position");out.push(...lines.slice(cursor,start));cursor=start;
    while(i<ds.length&&!ds[i].startsWith("@@ ")){
      const kind=ds[i][0],text=ds[i++].slice(1);
      ensure([" ","+","-"].includes(kind),"diff_line");
      if(kind===" "||kind==="+"){ensure(lines[cursor]===text,"diff_exact_line");cursor++;}
      if(kind===" "||kind==="-")out.push(text);
    }
  }
  ensure(hunks===4,"diff_hunk_count");out.push(...lines.slice(cursor));return out.join("");
}
let CANDIDATE,BASELINE,SCRIPT,COLLECTORS;
function bindSource(){
  CANDIDATE=decode(BINDING.candidate_b64);BASELINE=decode(BINDING.baseline_b64);
  const diff=decode(BINDING.diff_b64);
  ensure(Buffer.byteLength(CANDIDATE)===32502&&sha(CANDIDATE)===BINDING.candidate_sha256,
    "candidate_identity");
  ensure(Buffer.byteLength(BASELINE)===31581&&sha(BASELINE)===BINDING.baseline_sha256,
    "baseline_identity");
  ensure(Buffer.byteLength(diff)===2243&&sha(diff)===BINDING.diff_sha256,"diff_identity");
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
      {role:"statictext",name:"Use Sign in to open this local application."}]:
    [{role:"textbox",name:"Username"},{role:"textbox",name:"Password"},
     {role:"button",name:"Sign in"}];
  return {structuredContent:{status:"ok",snapshot:{complete:true},content_refs:nodes,
    refs:[{ref,actions:["click"]}]}};
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
      if(kind==="missing")return {structuredContent:{status:"ok",snapshot:{complete:true},
        content_refs:[],refs:[]}};
      return publicPage(kind,ref);
    },
    mcp__cua_driver__browser_click:async a=>{
      bound(s,a,true);ensure(a.input_route==="dom_event","click_route");
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

### Complete future controller and payload assembly — NOT EXECUTED

Its assemble_payload function decodes the complete embedded payload and checks
132725 bytes / SHA256 46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866
before any child launch. The full folded base64 literal below is DATA, exact
to the full readable Node program above. No archived original browser
controller/collector is imported or run.

```python
# Future ONE memory-only controller; NOT executed in this source-design phase.
import time
START_NS=time.monotonic_ns()
import base64,hashlib,json,math,os,pathlib,re,selectors,shutil,signal,stat,subprocess,sys
ROOT=pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27")
NODE=pathlib.Path("/opt/homebrew/Cellar/node/26.7.0/bin/node")
NODE_SHA="1ef99ea25fe70c9b67e7efe768ef8ee22148d3cabc703db6131b57aeb617d040"
PAYLOAD_SHA="46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866"
PAYLOAD_BYTES=132725
PAYLOAD_B64=(
    "aW1wb3J0IHZtIGZyb20gIm5vZGU6dm0iOwppbXBvcnQge2NyZWF0ZUhhc2h9IGZyb20gIm5vZGU6Y3J5cHRvIjsKaW1wb3J0"
    "IHtwZXJmb3JtYW5jZX0gZnJvbSAibm9kZTpwZXJmX2hvb2tzIjsKY29uc3QgQklORElORyA9IHsKICAiY2FuZGlkYXRlX2I2"
    "NCI6ICJMeThnVUhKdmMzQmxZM1JwZG1VZ1QwNUZJR1oxYm1OMGFXOXVjeTVsZUdWaklHTmxiR3c3SUU1UFZDQmxlR1ZqZFhS"
    "bFpDQnBiaUIwYUdseklHUmxjMmxuYmlCd2FHRnpaUzRLWTI5dWMzUWdRMHhQUTBzOUltbHRjRzl5ZENCcWMyOXVMSFJwYldW"
    "Y2JuQnlhVzUwS0dwemIyNHVaSFZ0Y0hNb2V5ZHRiMjV2ZEc5dWFXTmZibk1uT25OMGNpaDBhVzFsTG0xdmJtOTBiMjVwWTE5"
    "dWN5Z3BLU3duZDJGc2JGOWxjRzlqYUY5dWN5YzZjM1J5S0hScGJXVXVkR2x0WlY5dWN5Z3BLWDBwS1Z4dUlqc0tZMjl1YzNR"
    "Z1UxUlBVRDBpYVcxd2IzSjBJR3B6YjI0c2IzTXNjR0YwYUd4cFlpeHpkR0YwTEhONWN5eDBhVzFsWEc1alBXcHpiMjR1Ykc5"
    "aFpITW9jM2x6TG1GeVozWmJNVjBwWEc1amJHOWphejE3SjIxdmJtOTBiMjVwWTE5dWN5YzZjM1J5S0hScGJXVXViVzl1YjNS"
    "dmJtbGpYMjV6S0NrcExDZDNZV3hzWDJWd2IyTm9YMjV6SnpwemRISW9kR2x0WlM1MGFXMWxYMjV6S0NrcGZWeHVjbVZqYjNK"
    "a1BYc25jMk5vWlcxaEp6b25jbWxoZFhSb0xtUXdNUzFtYVhKemRDMXZZbk5sY25aaGRHbHZiaTkyTVNjc0oyWnBjbk4wWDJa"
    "aGFXeDFjbVVuT21OYkoyWnBjbk4wWDJaaGFXeDFjbVVuWFN3blptbHljM1JmYjJKelpYSjJZWFJwYjI1ZmQyRnNiRjl0Y3lj"
    "Nlkxc25abWx5YzNSZmIySnpaWEoyWVhScGIyNWZkMkZzYkY5dGN5ZGRMQ2RtYVhKemRGOWxkbVZ1ZEY5d2NtOTJaVzRuT2ta"
    "aGJITmxMQ2RqYkc5amF5YzZZMnh2WTJ0OVhHNWxkbVZ1ZEY5emRHRjBaVDBuZDNKcGRHVmZkVzVqYjI1bWFYSnRaV1FuWEc1"
    "MGNuazZYRzRnSUNBZ1ptUTliM011YjNCbGJpaHdZWFJvYkdsaUxsQmhkR2dvWTFzblpYWmxiblJmYjNWMEoxMHBMRzl6TGs5"
    "ZlYxSlBUa3haZkc5ekxrOWZRMUpGUVZSOGIzTXVUMTlGV0VOTWZHOXpMazlmVGs5R1QweE1UMWNzTUc4Mk1EQXBYRzRnSUNB"
    "Z2QybDBhQ0J2Y3k1bVpHOXdaVzRvWm1Rc0ozY25MR1Z1WTI5a2FXNW5QU2RoYzJOcGFTY3BJR0Z6SUdZNlhHNGdJQ0FnSUNB"
    "Z0lHWXVkM0pwZEdVb2FuTnZiaTVrZFcxd2N5aHlaV052Y21Rc2MyOXlkRjlyWlhselBWUnlkV1VwS3lkY1hHNG5LVHRtTG1a"
    "c2RYTm9LQ2s3YjNNdVpuTjVibU1vWmk1bWFXeGxibThvS1NsY2JpQWdJQ0JsZG1WdWRGOXpkR0YwWlQwbmQzSnBkSFJsYmlk"
    "Y2JtVjRZMlZ3ZENCUFUwVnljbTl5T25CaGMzTmNiaU1nUVhSMFpXMXdkQ0JqYkc5amF5QndaWEp6YVhOMFpXNWpaU0JDUlVa"
    "UFVrVWdiV0Z5YTJWeUwzTjBZWFJsTDJKMVpHZGxkQ0JqYjIxd1lYSnBjMjl1Y3k1Y2JpTWdRU0J0WlhSaFpHRjBZU0JsY25K"
    "dmNpQmtiMlZ6SUc1dmRDQndjbVYyWlc1MElIUm9aU0J2Y21sbmFXNWhiQ0JsYzNObGJuUnBZV3dnYzNSdmNDQndjbTkwYjJO"
    "dmJDNWNibXhoWWoxd1lYUm9iR2xpTGxCaGRHZ29ZMXNuYkdGaUoxMHBPMjFoY210bGNsOXpkR0YwWlQwbmJHRmlYMkZpYzJW"
    "dWRDZGNiblJ5ZVRwY2JpQWdJQ0JwWmlCc1lXSXVaWGhwYzNSektDazZYRzRnSUNBZ0lDQWdJR2x1Wm04OWJHRmlMbXh6ZEdG"
    "MEtDbGNiaUFnSUNBZ0lDQWdhV1lnYm05MElITjBZWFF1VTE5SlUwUkpVaWhwYm1adkxuTjBYMjF2WkdVcElHOXlJSE4wWVhR"
    "dVUxOUpUVTlFUlNocGJtWnZMbk4wWDIxdlpHVXBJVDB3Ynpjd01DQnZjaUJwYm1adkxuTjBYM1ZwWkNFOWIzTXVaMlYwZFds"
    "a0tDazZYRzRnSUNBZ0lDQWdJQ0FnSUNCdFlYSnJaWEpmYzNSaGRHVTlKMjkzYm1WeWMyaHBjRjkxYm10dWIzZHVKMXh1SUNB"
    "Z0lDQWdJQ0JsYkhObE9seHVJQ0FnSUNBZ0lDQWdJQ0FnYldGeWEyVnlYM04wWVhSbFBTZHpkRzl3WDNKbGNYVmxjM1JsWkNk"
    "Y2JpQWdJQ0FnSUNBZ0lDQWdJR1p2Y2lCdVlXMWxJR2x1SUNnb0ozVnBMV1poYVd4MWNtVW5MQ2R6ZEc5d0p5a2dhV1lnWTFz"
    "blptbHljM1JmWm1GcGJIVnlaU2RkSUdseklHNXZkQ0JPYjI1bElHVnNjMlVnS0NkemRHOXdKeXdwS1RwY2JpQWdJQ0FnSUNB"
    "Z0lDQWdJQ0FnSUNCMGNuazZYRzRnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUdaa1BXOXpMbTl3Wlc0b2JHRmlMMjVoYldV"
    "c2IzTXVUMTlYVWs5T1RGbDhiM011VDE5RFVrVkJWSHh2Y3k1UFgwVllRMHg4YjNNdVQxOU9UMFpQVEV4UFZ5d3diell3TUNs"
    "Y2JpQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdiM011WTJ4dmMyVW9abVFwWEc0Z0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnWlho"
    "alpYQjBJRVpwYkdWT2IzUkdiM1Z1WkVWeWNtOXlPbTFoY210bGNsOXpkR0YwWlQwbmJHRmlYMkZpYzJWdWRDZGNiaUFnSUNB"
    "Z0lDQWdJQ0FnSUNBZ0lDQmxlR05sY0hRZ1JtbHNaVVY0YVhOMGMwVnljbTl5T25CaGMzTmNibVY0WTJWd2RDQlBVMFZ5Y205"
    "eU9tMWhjbXRsY2w5emRHRjBaVDBuYzNSdmNGOTFibU52Ym1acGNtMWxaQ2RjYm5CeWFXNTBLR3B6YjI0dVpIVnRjSE1vZXlk"
    "amJHOWpheWM2WTJ4dlkyc3NKMlYyWlc1MFgzTjBZWFJsSnpwbGRtVnVkRjl6ZEdGMFpTd25iV0Z5YTJWeVgzTjBZWFJsSnpw"
    "dFlYSnJaWEpmYzNSaGRHVjlLU2xjYmlJN0NtTnZibk4wSUZKRlFVUkNRVU5MUFNKcGJYQnZjblFnYW5OdmJpeHZjeXh3WVhS"
    "b2JHbGlMSE4wWVhRc2MzVmljSEp2WTJWemN5eHplWE1zZEdsdFpWeHVZejFxYzI5dUxteHZZV1J6S0hONWN5NWhjbWQyV3pG"
    "ZEtWeHVZMmhwYkdSeVpXNDlUbTl1WlR0b1pXeHdaWEpmY0dsa1BXTmJKMmhsYkhCbGNsOXdhV1FuWFR0dmRYUmxjbDl2WW5O"
    "bGNuWmhkR2x2YmoxT2IyNWxPM0J5YjJwbFkzUnBiMjQ5SjNWdVlYWmhhV3hoWW14bEoxeHVaR1ZtSUdacGJtbDBaVjl2WW5O"
    "bGNuWmhkR2x2YmloMllXeDFaU2s2WEc0Z0lDQWdhV1lnZEhsd1pTaDJZV3gxWlNrZ2FYTWdibTkwSUdScFkzUWdiM0lnYzJW"
    "MEtIWmhiSFZsS1NFOUlIc25iMkp6WlhKMlpXUW5MQ2RrYVdGbmJtOXpkR2xqSjMwZ2IzSWdkSGx3WlNoMllXeDFaVnNuYjJK"
    "elpYSjJaV1FuWFNrZ2FYTWdibTkwSUdKdmIydzZYRzRnSUNBZ0lDQWdJSEpsZEhWeWJpQk9iMjVsWEc0Z0lDQWdaR2xoWjI1"
    "dmMzUnBZejEyWVd4MVpWc25aR2xoWjI1dmMzUnBZeWRkWEc0Z0lDQWdhV1lnWkdsaFoyNXZjM1JwWXlCcGN5Qk9iMjVsT2x4"
    "dUlDQWdJQ0FnSUNCeVpYUjFjbTRnZXlkdlluTmxjblpsWkNjNmRtRnNkV1ZiSjI5aWMyVnlkbVZrSjEwc0oyUnBZV2R1YjNO"
    "MGFXTW5PazV2Ym1WOVhHNGdJQ0FnYVdZZ2JtOTBJSFpoYkhWbFd5ZHZZbk5sY25abFpDZGRJRzl5SUhSNWNHVW9aR2xoWjI1"
    "dmMzUnBZeWtnYVhNZ2JtOTBJR1JwWTNRZ2IzSWdjMlYwS0dScFlXZHViM04wYVdNcElUMGdleWR6YVhSbEp5d25aWGhqWlhC"
    "MGFXOXVYMk5zWVhOekp5d25iM2R1WDJaMWJtTjBhVzl1Snl3bmIzZHVYMnhwYm1VbmZUcGNiaUFnSUNBZ0lDQWdjbVYwZFhK"
    "dUlFNXZibVZjYmlBZ0lDQnphWFJsY3owb0oyaGhibVJzWlhJbkxDZHpaWEoyWlhJbkxDZHRZV2x1SnlsY2JpQWdJQ0JyYVc1"
    "a2N6MG9KMEYwZEhKcFluVjBaVVZ5Y205eUp5d25WSGx3WlVWeWNtOXlKeXduVm1Gc2RXVkZjbkp2Y2ljc0owdGxlVVZ5Y205"
    "eUp5d25UMU5GY25KdmNpY3NKMEp5YjJ0bGJsQnBjR1ZGY25KdmNpY3NKME52Ym01bFkzUnBiMjVTWlhObGRFVnljbTl5Snl3"
    "blZHbHRaVzkxZEVWeWNtOXlKeXduYjNSb1pYSW5LVnh1SUNBZ0lHWjFibU4wYVc5dWN6MG9KMGhsWVdSbGNsSmxZV1JsY2k1"
    "eVpXRmtiR2x1WlNjc0owUmxiVzlUWlhKMlpYSXVjSEp2WTJWemMxOXlaWEYxWlhOMEp5d25SR1Z0Ynk1aVpXZHBiaWNzSjBS"
    "bGJXOHVZMkZzYkdKaFkyc25MQ2RFWlcxdkxtbHVkbTlyWlNjc0owaGhibVJzWlhJdWFHRnVaR3hsWDI5dVpWOXlaWEYxWlhO"
    "MEp5d25TR0Z1Wkd4bGNpNXpaVzVrWDJWeWNtOXlKeXduU0dGdVpHeGxjaTVuWlhRbkxDZElZVzVrYkdWeUxuSmxjR3g1Snl3"
    "bmJXRnBiaWNwWEc0Z0lDQWdjMmwwWlN4cmFXNWtMR1oxYm1OMGFXOXVMR3hwYm1VOUtHUnBZV2R1YjNOMGFXTmJhMTBnWm05"
    "eUlHc2dhVzRnS0NkemFYUmxKeXduWlhoalpYQjBhVzl1WDJOc1lYTnpKeXduYjNkdVgyWjFibU4wYVc5dUp5d25iM2R1WDJ4"
    "cGJtVW5LU2xjYmlBZ0lDQnBaaUIwZVhCbEtITnBkR1VwSUdseklHNXZkQ0J6ZEhJZ2IzSWdjMmwwWlNCdWIzUWdhVzRnYzJs"
    "MFpYTWdiM0lnZEhsd1pTaHJhVzVrS1NCcGN5QnViM1FnYzNSeUlHOXlJR3RwYm1RZ2JtOTBJR2x1SUd0cGJtUnpPbHh1SUNB"
    "Z0lDQWdJQ0J5WlhSMWNtNGdUbTl1WlZ4dUlDQWdJR2xtSUc1dmRDQW9LR1oxYm1OMGFXOXVJR2x6SUU1dmJtVWdZVzVrSUd4"
    "cGJtVWdhWE1nVG05dVpTa2diM0lnS0hSNWNHVW9ablZ1WTNScGIyNHBJR2x6SUhOMGNpQmhibVFnWm5WdVkzUnBiMjRnYVc0"
    "Z1puVnVZM1JwYjI1eklHRnVaQ0IwZVhCbEtHeHBibVVwSUdseklHbHVkQ0JoYm1RZ01UdzliR2x1WlR3OU1UQXlOQ2twT2x4"
    "dUlDQWdJQ0FnSUNCeVpYUjFjbTRnVG05dVpWeHVJQ0FnSUhKbGRIVnliaUI3SjI5aWMyVnlkbVZrSnpwVWNuVmxMQ2RrYVdG"
    "bmJtOXpkR2xqSnpwN0ozTnBkR1VuT25OcGRHVXNKMlY0WTJWd2RHbHZibDlqYkdGemN5YzZhMmx1WkN3bmIzZHVYMloxYm1O"
    "MGFXOXVKenBtZFc1amRHbHZiaXduYjNkdVgyeHBibVVuT214cGJtVjlmVnh1YjNWMFpYSTljR0YwYUd4cFlpNVFZWFJvS0dO"
    "YkoyOTFkR1Z5WDI5MWRDZGRLVnh1YVdZZ2IzVjBaWEl1YVhOZlptbHNaU2dwT2x4dUlDQWdJR1prUFc5ekxtOXdaVzRvYjNW"
    "MFpYSXNiM011VDE5U1JFOU9URmw4YjNNdVQxOU9UMFpQVEV4UFYzeHZjeTVQWDA1UFRrSk1UME5MS1Z4dUlDQWdJSFJ5ZVRw"
    "Y2JpQWdJQ0FnSUNBZ2FXNW1iejF2Y3k1bWMzUmhkQ2htWkNsY2JpQWdJQ0FnSUNBZ2FXWWdibTkwSUNoemRHRjBMbE5mU1ZO"
    "U1JVY29hVzVtYnk1emRGOXRiMlJsS1NCaGJtUWdhVzVtYnk1emRGOTFhV1E5UFc5ekxtZGxkSFZwWkNncElHRnVaQ0J6ZEdG"
    "MExsTmZTVTFQUkVVb2FXNW1ieTV6ZEY5dGIyUmxLVDA5TUc4Mk1EQWdZVzVrSURBOGFXNW1ieTV6ZEY5emFYcGxQRDB5TmpJ"
    "eE5EUXBPbHh1SUNBZ0lDQWdJQ0FnSUNBZ2NtRnBjMlVnVm1Gc2RXVkZjbkp2Y2lnblptbDRaV1JmYjNWMFpYSmZaWFpwWkdW"
    "dVkyVmZhVzUyWVd4cFpDY3BYRzRnSUNBZ0lDQWdJSGRwZEdnZ2IzTXVabVJ2Y0dWdUtHWmtMQ2R5WWljc1kyeHZjMlZtWkQx"
    "R1lXeHpaU2tnWVhNZ2MyOTFjbU5sT25KaGQxOWllWFJsY3oxemIzVnlZMlV1Y21WaFpDZ3lOakl4TkRVcFhHNGdJQ0FnSUNB"
    "Z0lHbG1JRzV2ZENBd1BHeGxiaWh5WVhkZllubDBaWE1wUEQweU5qSXhORFE2Y21GcGMyVWdWbUZzZFdWRmNuSnZjaWduWm1s"
    "NFpXUmZiM1YwWlhKZlpYWnBaR1Z1WTJWZmFXNTJZV3hwWkNjcFhHNGdJQ0FnSUNBZ0lHUmhkR0U5YW5OdmJpNXNiMkZrY3lo"
    "eVlYZGZZbmwwWlhNcFhHNGdJQ0FnWm1sdVlXeHNlVHB2Y3k1amJHOXpaU2htWkNsY2JpQWdJQ0J2ZFhSbGNsOXZZbk5sY25a"
    "aGRHbHZiajFtYVc1cGRHVmZiMkp6WlhKMllYUnBiMjRvWkdGMFlTNW5aWFFvSjNWdVpYaHdaV04wWldSZlptRnBiSFZ5WlY5"
    "dlluTmxjblpoZEdsdmJpY3BLVnh1SUNBZ0lIQnliMnBsWTNScGIyNDlaR0YwWVM1blpYUW9KM1Z1Wlhod1pXTjBaV1JmYjJK"
    "elpYSjJZWFJwYjI1ZmNISnZhbVZqZEdsdmJpY3BYRzRnSUNBZ2FXWWdjSEp2YW1WamRHbHZiaUJ1YjNRZ2FXNGdLQ2QxYm1G"
    "MllXbHNZV0pzWlNjc0ozWmhiR2xrSnl3bmFXNTJZV3hwWkNjcE9uQnliMnBsWTNScGIyNDlKMmx1ZG1Gc2FXUW5YRzRnSUNB"
    "Z2FXWWdjSEp2YW1WamRHbHZiajA5SjNaaGJHbGtKeUJoYm1RZ2IzVjBaWEpmYjJKelpYSjJZWFJwYjI0Z2FYTWdUbTl1WlRw"
    "d2NtOXFaV04wYVc5dVBTZHBiblpoYkdsa0oxeHVJQ0FnSUdGc2JHOTNaV1E5ZXlkM2FHOWhiV2tuTENka2FYTmpiM1psY25r"
    "bkxDZGpiMjVtYVdSbGJuUnBZV3hmWTJ4cFpXNTBYMk55WldGMFpTY3NKMjl3WlhKaGRHOXlYMnh2WjJsdUp5d25jMlZ5ZG1W"
    "eUp5d25iV0ZwYm5SbGJtRnVZMlZmYVc1cGRDZDlYRzRnSUNBZ2MzUmhjblJsWkQxa1lYUmhMbWRsZENnbmFHVnNjR1Z5WDJs"
    "dWRtOWpZWFJwYjI1ekp5azlQVEVnWVc1a0lIUjVjR1VvWkdGMFlTNW5aWFFvSjJobGJIQmxjbDl3YVdRbktTa2dhWE1nYVc1"
    "MElHRnVaQ0JrWVhSaFd5ZG9aV3h3WlhKZmNHbGtKMTArTUZ4dUlDQWdJR2xtSUhOMFlYSjBaV1E2WVd4c2IzZGxaQzVoWkdR"
    "b0oyaGxiSEJsY2ljcFhHNGdJQ0FnY21GM1BXUmhkR0V1WjJWMEtDZHZkMjVsWkY5amFHbHNaRjlsZUdsMGN5Y3BYRzRnSUNB"
    "Z2FXWWdhWE5wYm5OMFlXNWpaU2h5WVhjc2JHbHpkQ2tnWVc1a0lHeGxiaWh5WVhjcFBUMXNaVzRvWVd4c2IzZGxaQ2tnWVc1"
    "a0lHRnNiQ2hwYzJsdWMzUmhibU5sS0hZc1pHbGpkQ2tnWVc1a0lIWXVaMlYwS0NkdVlXMWxKeWtnYVc0Z1lXeHNiM2RsWkNC"
    "aGJtUWdkSGx3WlNoMkxtZGxkQ2duY0dsa0p5a3BJR2x6SUdsdWRDQmhibVFnZGxzbmNHbGtKMTArTUNCaGJtUWdkSGx3WlNo"
    "MkxtZGxkQ2duWlhocGRDY3BLU0JwY3lCcGJuUWdabTl5SUhZZ2FXNGdjbUYzS1NCaGJtUWdlM1piSjI1aGJXVW5YU0JtYjNJ"
    "Z2RpQnBiaUJ5WVhkOVBUMWhiR3h2ZDJWa0lHRnVaQ0JzWlc0b2UzWmJKM0JwWkNkZElHWnZjaUIySUdsdUlISmhkMzBwUFQx"
    "c1pXNG9ZV3hzYjNkbFpDa2dZVzVrSUc1bGVIUW9kbHNuY0dsa0oxMGdabTl5SUhZZ2FXNGdjbUYzSUdsbUlIWmJKMjVoYldV"
    "blhUMDlKM05sY25abGNpY3BQVDFqV3lkelpYSjJaWEpmY0dsa0oxMGdZVzVrSUNnb2MzUmhjblJsWkNCaGJtUWdibVY0ZENo"
    "Mld5ZHdhV1FuWFNCbWIzSWdkaUJwYmlCeVlYY2dhV1lnZGxzbmJtRnRaU2RkUFQwbmFHVnNjR1Z5SnlrOVBXUmhkR0ZiSjJo"
    "bGJIQmxjbDl3YVdRblhTQmhibVFnS0dobGJIQmxjbDl3YVdRZ2FYTWdUbTl1WlNCdmNpQm9aV3h3WlhKZmNHbGtQVDFrWVhS"
    "aFd5ZG9aV3h3WlhKZmNHbGtKMTBwS1NCdmNpQW9ibTkwSUhOMFlYSjBaV1FnWVc1a0lHaGxiSEJsY2w5d2FXUWdhWE1nVG05"
    "dVpTQmhibVFnWkdGMFlTNW5aWFFvSjJobGJIQmxjbDlwYm5adlkyRjBhVzl1Y3ljcElHbHpJRTV2Ym1VZ1lXNWtJR1JoZEdF"
    "dVoyVjBLQ2RvWld4d1pYSmZjR2xrSnlrZ2FYTWdUbTl1WlNrcE9seHVJQ0FnSUNBZ0lDQmphR2xzWkhKbGJqMWJlMnM2ZGx0"
    "clhTQm1iM0lnYXlCcGJpQW9KMjVoYldVbkxDZHdhV1FuTENkbGVHbDBKeWw5SUdadmNpQjJJR2x1SUhKaGQxMWNiaUFnSUNB"
    "Z0lDQWdhR1ZzY0dWeVgzQnBaRDFrWVhSaFd5ZG9aV3h3WlhKZmNHbGtKMTBnYVdZZ2MzUmhjblJsWkNCbGJITmxJRTV2Ym1W"
    "Y2JuQnBaSE05YkdsemRDaGpXeWR2ZDI1bFpGOXdhV1J6SjEwcFhHNXBaaUJvWld4d1pYSmZjR2xrSUdseklHNXZkQ0JPYjI1"
    "bElHRnVaQ0JvWld4d1pYSmZjR2xrSUc1dmRDQnBiaUJ3YVdSek9uQnBaSE11WVhCd1pXNWtLR2hsYkhCbGNsOXdhV1FwWEc1"
    "d1BYTjFZbkJ5YjJObGMzTXVjblZ1S0ZzbkwySnBiaTl3Y3ljc0p5MXdKeXduTENjdWFtOXBiaWh6ZEhJb2Rpa2dabTl5SUhZ"
    "Z2FXNGdjR2xrY3lrc0p5MXZKeXduY0dsa1BTZGRMR05oY0hSMWNtVmZiM1YwY0hWMFBWUnlkV1VzZEdsdFpXOTFkRDB6S1Z4"
    "dWNITmZhMjV2ZDI0OWNDNXlaWFIxY201amIyUmxJR2x1SUNnd0xERXBJR0Z1WkNCaGJHd29kaTVwYzJScFoybDBLQ2tnWm05"
    "eUlIWWdhVzRnY0M1emRHUnZkWFF1YzNCc2FYUW9LU2xjYm5CeVpYTmxiblE5YzJWMEtHbHVkQ2gyS1NCbWIzSWdkaUJwYmlC"
    "d0xuTjBaRzkxZEM1emNHeHBkQ2dwS1NCcFppQndjMTlyYm05M2JpQmxiSE5sSUhObGRDZ3BYRzV3YjNKMGN6MTdmVnh1Wm05"
    "eUlIQnZjblFnYVc0Z0tEa3dNREFzTXpBd01DazZYRzRnSUNBZ2NEMXpkV0p3Y205alpYTnpMbkoxYmloYkp5OTFjM0l2YzJK"
    "cGJpOXNjMjltSnl3bkxXNVFKeXduTFhRbkxDY3RhVlJEVURvbkszTjBjaWh3YjNKMEtTd25MWE5VUTFBNlRFbFRWRVZPSjEw"
    "c1kyRndkSFZ5WlY5dmRYUndkWFE5VkhKMVpTeDBhVzFsYjNWMFBUTXBYRzRnSUNBZ2NHOXlkSE5iYzNSeUtIQnZjblFwWFQx"
    "dWIzUWdZbTl2YkNod0xuTjBaRzkxZEM1emRISnBjQ2dwS1NCcFppQndMbkpsZEhWeWJtTnZaR1VnYVc0Z0tEQXNNU2tnWld4"
    "elpTQk9iMjVsWEc1d2NtbHVkQ2hxYzI5dUxtUjFiWEJ6S0hzblkyeHZZMnNuT25zbmJXOXViM1J2Ym1salgyNXpKenB6ZEhJ"
    "b2RHbHRaUzV0YjI1dmRHOXVhV05mYm5Nb0tTa3NKM2RoYkd4ZlpYQnZZMmhmYm5Nbk9uTjBjaWgwYVcxbExuUnBiV1ZmYm5N"
    "b0tTbDlMQ2R2ZDI1bFpGOXdhV1J6SnpwN2MzUnlLSFlwT2loMklHNXZkQ0JwYmlCd2NtVnpaVzUwSUdsbUlIQnpYMnR1YjNk"
    "dUlHVnNjMlVnVG05dVpTa2dabTl5SUhZZ2FXNGdjR2xrYzMwc0ozQnZjblJ6Snpwd2IzSjBjeXduYkdGaVgyRmljMlZ1ZENj"
    "NmJtOTBJSEJoZEdoc2FXSXVVR0YwYUNoald5ZHNZV0luWFNrdVpYaHBjM1J6S0Nrc0oyOTNibVZrWDJOb2FXeGtYMlY0YVhS"
    "ekp6cGphR2xzWkhKbGJpd25hR1ZzY0dWeVgzQnBaQ2M2YUdWc2NHVnlYM0JwWkN3bmRXNWxlSEJsWTNSbFpGOW1ZV2xzZFhK"
    "bFgyOWljMlZ5ZG1GMGFXOXVKenB2ZFhSbGNsOXZZbk5sY25aaGRHbHZiaXduZFc1bGVIQmxZM1JsWkY5dlluTmxjblpoZEds"
    "dmJsOXdjbTlxWldOMGFXOXVKenB3Y205cVpXTjBhVzl1ZlNrcFhHNGlPd3BqYjI1emRDQk5RVkpMUlZJOUltbHRjRzl5ZENC"
    "dmN5eHdZWFJvYkdsaUxITjBZWFFzYzNselhHNXNZV0k5Y0dGMGFHeHBZaTVRWVhSb0tITjVjeTVoY21kMld6RmRLVHRuZFdG"
    "eVpEMXBiblFvYzNsekxtRnlaM1piTWwwcE8zTmxjblpsY2oxcGJuUW9jM2x6TG1GeVozWmJNMTBwWEc1cFppQm5kV0Z5WkR3"
    "OU1DQnZjaUJ6WlhKMlpYSThQVEFnYjNJZ1ozVmhjbVE5UFhObGNuWmxjanB5WVdselpTQldZV3gxWlVWeWNtOXlLQ2R2ZDI1"
    "bFpGOWpiMjUwWlhoMFgybHVkbUZzYVdRbktWeHVhVzVtYnoxc1lXSXViSE4wWVhRb0tWeHVhV1lnYm05MElDaHpkR0YwTGxO"
    "ZlNWTkVTVklvYVc1bWJ5NXpkRjl0YjJSbEtTQmhibVFnYzNSaGRDNVRYMGxOVDBSRktHbHVabTh1YzNSZmJXOWtaU2s5UFRC"
    "dk56QXdJR0Z1WkNCcGJtWnZMbk4wWDNWcFpEMDliM011WjJWMGRXbGtLQ2twT25KaGFYTmxJRlpoYkhWbFJYSnliM0lvSjI5"
    "M2JtVmtYMk52Ym5SbGVIUmZhVzUyWVd4cFpDY3BYRzV2Y3k1cmFXeHNLR2QxWVhKa0xEQXBPMjl6TG10cGJHd29jMlZ5ZG1W"
    "eUxEQXBYRzVwWmlBb2JHRmlMeWR6ZEc5d0p5a3VaWGhwYzNSektDa2diM0lnS0d4aFlpOG5kV2t0Wm1GcGJIVnlaU2NwTG1W"
    "NGFYTjBjeWdwT25KaGFYTmxJRlpoYkhWbFJYSnliM0lvSjI5M2JtVmtYMk52Ym5SbGVIUmZhVzUyWVd4cFpDY3BYRzVtWkQx"
    "dmN5NXZjR1Z1S0d4aFlpOG5Zbkp2ZDNObGNpMXdjbVZ3WVhKbFpDY3NiM011VDE5WFVrOU9URmw4YjNNdVQxOURVa1ZCVkh4"
    "dmN5NVBYMFZZUTB4OGIzTXVUMTlPVDBaUFRFeFBWeXd3YnpZd01DbGNibTl6TG1Oc2IzTmxLR1prS1Z4dWNISnBiblFvSjN0"
    "Y0luQnlaWEJoY21Wa1gyMWhjbXRsY2w5M2NtbDBkR1Z1WENJNmRISjFaWDBuS1Z4dUlqc0tZMjl1YzNRZ1VFVlNVMGxUVkQw"
    "aWFXMXdiM0owSUdwemIyNHNiM01zY0dGMGFHeHBZaXh6ZVhOY2JuQmhkR2c5Y0dGMGFHeHBZaTVRWVhSb0tITjVjeTVoY21k"
    "Mld6RmRLVnh1Y21WamIzSmtQV3B6YjI0dWJHOWhaSE1vYzNsekxtRnlaM1piTWwwcFhHNGpJRU52Ym5abGNuUWdaR1ZqYVcx"
    "aGJDQnpkSEpwYm1keklHUnBjbVZqZEd4NUlIUnZJRkI1ZEdodmJpQnBiblJsWjJWeWN5d2dZWFp2YVdScGJtY2dTbE1nVG5W"
    "dFltVnlJSEp2ZFc1a2FXNW5MbHh1WkdWbUlHTnNiMk5yY3loMllXeDFaU2s2WEc0Z0lDQWdhV1lnYVhOcGJuTjBZVzVqWlNo"
    "MllXeDFaU3hrYVdOMEtUcGNiaUFnSUNBZ0lDQWdabTl5SUdzc2RpQnBiaUJzYVhOMEtIWmhiSFZsTG1sMFpXMXpLQ2twT2x4"
    "dUlDQWdJQ0FnSUNBZ0lDQWdhV1lnYXlCcGJpQW9KMjF2Ym05MGIyNXBZMTl1Y3ljc0ozZGhiR3hmWlhCdlkyaGZibk1uS1NC"
    "aGJtUWdhWE5wYm5OMFlXNWpaU2gyTEhOMGNpazZYRzRnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdZWE56WlhKMElIWXVhWE5rWldO"
    "cGJXRnNLQ2tnWVc1a0lHeGxiaWgyS1R3OU1Ua2dZVzVrSURBOFBXbHVkQ2gyS1R3eUtpbzJNMXh1SUNBZ0lDQWdJQ0FnSUNB"
    "Z0lDQWdJSFpoYkhWbFcydGRQV2x1ZENoMktWeHVJQ0FnSUNBZ0lDQWdJQ0FnWld4elpUcGpiRzlqYTNNb2RpbGNiaUFnSUNC"
    "bGJHbG1JR2x6YVc1emRHRnVZMlVvZG1Gc2RXVXNiR2x6ZENrNlhHNGdJQ0FnSUNBZ0lHWnZjaUIySUdsdUlIWmhiSFZsT21O"
    "c2IyTnJjeWgyS1Z4dVkyeHZZMnR6S0hKbFkyOXlaQ2xjYm1aa1BXOXpMbTl3Wlc0b2NHRjBhQ3h2Y3k1UFgxZFNUMDVNV1h4"
    "dmN5NVBYME5TUlVGVWZHOXpMazlmUlZoRFRIeHZjeTVQWDA1UFJrOU1URTlYTERCdk5qQXdLVnh1ZDJsMGFDQnZjeTVtWkc5"
    "d1pXNG9abVFzSjNjbkxHVnVZMjlrYVc1blBTZGhjMk5wYVNjcElHRnpJR1k2WEc0Z0lDQWdaaTUzY21sMFpTaHFjMjl1TG1S"
    "MWJYQnpLSEpsWTI5eVpDeHpiM0owWDJ0bGVYTTlWSEoxWlN4cGJtUmxiblE5TWlrckoxeGNiaWNwTzJZdVpteDFjMmdvS1R0"
    "dmN5NW1jM2x1WXlobUxtWnBiR1Z1YnlncEtWeHVjSEpwYm5Rb2FuTnZiaTVrZFcxd2N5aDdKM2R5YVhSMFpXNWZaWGhqYkhW"
    "emFYWmxKenBVY25WbGZTa3BYRzRpT3dwamIyNXpkQ0J2ZDI0OWJHOWhaQ2dpWkRBeFgybHRiV1ZrYVdGMFpWOXZkMjVsWkY5"
    "b1lXNWtiR1Z6SWlrN0NtTnZibk4wSUU5Q1UwdEZXVDBpWkRBeFgybHRiV1ZrYVdGMFpWOXZZbk5sY25aaGRHbHZibDl5WldO"
    "dmNtUWlPd3BwWmlBb0lXOTNiaUI4ZkNCdmQyNHVjblZ1ZEdsdFpWOXlaV3hsWVhObElUMDlkSEoxWlNCOGZDQnZkMjR1WkhK"
    "cGRtVnlYMjkzYm1Wa0lUMDlkSEoxWlNCOGZBb2dJQ0FnYjNkdUxtVjRZV04wWDJKdmRXNWtJVDA5ZEhKMVpTQjhmQ0J2ZDI0"
    "dWMybHVaMnhsWDJOdmJuUnliMnhzWlhJaFBUMTBjblZsSUh4OENpQWdJQ0FoV3lKd2NtVndZWEpsWkY5bGJuUnllU0lzSW1K"
    "eWIzZHpaWEpmWkdWamFYTnBiMjRpWFM1cGJtTnNkV1JsY3lodmQyNHVjR2hoYzJVcElIeDhDaUFnSUNBb2IzZHVMbkJvWVhO"
    "bFBUMDlJbkJ5WlhCaGNtVmtYMlZ1ZEhKNUlpWW1LRzkzYmk1d2NtVndZWEpsWkY5bllYUmxJVDA5ZEhKMVpYeDhiM2R1TG1o"
    "bGJIQmxjbDl3YVdRaFBUMXVkV3hzZkh3S0lDQWdJQ0FnYjNkdUxtWnBlSFIxY21WZmNtVmhaSGs5UFQxMGNuVmxmSHh2ZDI0"
    "dWJHbHpkR1Z1WlhKZmNHbGtYM0J5YjI5bVBUMDlkSEoxWlNrcElIeDhDaUFnSUNBb2IzZHVMbkJvWVhObFBUMDlJbUp5YjNk"
    "elpYSmZaR1ZqYVhOcGIyNGlKaVlvYjNkdUxtWnBlSFIxY21WZmNtVmhaSGtoUFQxMGNuVmxmSHh2ZDI0dWJHbHpkR1Z1WlhK"
    "ZmNHbGtYM0J5YjI5bUlUMDlkSEoxWlNrcElIeDhDaUFnSUNCdmQyNHVabkpsYzJoZmNHRjBhSE5mY0hKbFpteHBaMmgwSVQw"
    "OWRISjFaU0I4ZkFvZ0lDQWdJVTUxYldKbGNpNXBjMU5oWm1WSmJuUmxaMlZ5S0c5M2JpNXpkR0Z5ZEY5bGNHOWphRjl0Y3lr"
    "Z2ZId2dkSGx3Wlc5bUlHOTNiaTUzYjNKcmMzQmhZMlVoUFQwaWMzUnlhVzVuSWlCOGZBb2dJQ0FnYm1WM0lGTmxkQ2hiYjNk"
    "dUxtZDFZWEprWDNCcFpDeHZkMjR1YzJWeWRtVnlYM0JwWkN4dmQyNHVZbkp2ZDNObGNsOXdhV1FzQ2lBZ0lDQWdJQ0F1TGk0"
    "b2IzZHVMbWhsYkhCbGNsOXdhV1E5UFQxdWRXeHNQMXRkT2x0dmQyNHVhR1ZzY0dWeVgzQnBaRjBwWFNrdWMybDZaU0U5UFNo"
    "dmQyNHVhR1ZzY0dWeVgzQnBaRDA5UFc1MWJHdy9Nem8wS1NCOGZBb2dJQ0FnSVZ0dmQyNHVaM1ZoY21SZmNHbGtMRzkzYmk1"
    "elpYSjJaWEpmY0dsa0xHOTNiaTVpY205M2MyVnlYM0JwWkN4dmQyNHVaWGhsWTE5elpYTnphVzl1TEFvZ0lDQWdJQ0FnTGk0"
    "dUtHOTNiaTVvWld4d1pYSmZjR2xrUFQwOWJuVnNiRDliWFRwYmIzZHVMbWhsYkhCbGNsOXdhV1JkS1YwS0lDQWdJQ0FnSUM1"
    "bGRtVnllU2gyUFQ1T2RXMWlaWEl1YVhOVFlXWmxTVzUwWldkbGNpaDJLU1ltZGo0d0tTQjhmQW9nSUNBZ0lWdHZkMjR1YzJW"
    "emMybHZiaXh2ZDI0dWRHRnlaMlYwWDJsa0xHOTNiaTUwWVdKZmFXUXNiM2R1TG14aFlpeHZkMjR1YjNWMFpYSmZiM1YwTEFv"
    "Z0lDQWdJQ0FnYjNkdUxtVjJaVzUwWDI5MWRDeHZkMjR1WTJ4bFlXNTFjRjl2ZFhSZExtVjJaWEo1S0hZOVBuUjVjR1Z2WmlC"
    "MlBUMDlJbk4wY21sdVp5SW1Kbll1YkdWdVozUm9QakFwS1NCN0NpQWdkR1Y0ZENoN2NISnZjRzl6WVd4ZmNtVm1kWE5sWkRv"
    "aVpuSmxjMmhmYjNkdVpXUmZZMjl1ZEdWNGRGOXlaWEYxYVhKbFpDSjlLVHRsZUdsMEtDazdDbjBLWTI5dWMzUWdjSEpwYjNJ"
    "OWJHOWhaQ2hQUWxOTFJWa3BPd3BwWmlodmQyNHVZMnh2YzJWa1BUMDlkSEoxWlh4OGNISnBiM0kvTG1Oc1pXRnVkWEJmYzNS"
    "aGNuUmxaRDA5UFhSeWRXVXBld29nSUhSbGVIUW9lM0J5YjNCdmMyRnNYM0psWm5WelpXUTZJbVpwZUhSMWNtVmZZV3h5WldG"
    "a2VWOXpkRzl3Y0dWa0lpeHlaWE52ZFhKalpWOXlaV3hsWVhObFgzQnliM1psYmpwbVlXeHpaWDBwTzJWNGFYUW9LVHNLZlFw"
    "amIyNXpkQ0J5WldOdmNtUTljSEpwYjNJL1Azc0tJQ0J6WTJobGJXRTZJbkpwWVhWMGFDNWtNREV0YVcxdFpXUnBZWFJsTFc5"
    "aWMyVnlkbUYwYVc5dUxXTnNaV0Z1ZFhBdmRqRWlMQW9nSUdacGNuTjBYMlpoYVd4MWNtVTZiblZzYkN4bWFYSnpkRjl2WW5O"
    "bGNuWmhkR2x2Ymw5M1lXeHNYMjF6T201MWJHd3NDaUFnWm1seWMzUmZaWFpsYm5SZmNISnZkbVZ1T21aaGJITmxMSGRvYjJ4"
    "bFgyTnNaV0Z1ZFhCZmQybDBhR2x1TmpCZmNISnZkbVZ1T21aaGJITmxMQW9nSUhWdVpYaHdaV04wWldSZlptRnBiSFZ5WlY5"
    "dlluTmxjblpoZEdsdmJqcHVkV3hzTEdScFlXZHViM04wYVdOZlpYSnliM0p6T2x0ZExHTnNaV0Z1ZFhCZmMzUmhjblJsWkRw"
    "bVlXeHpaU3dLSUNCbWFYSnpkRjlqYkc5amF6cHVkV3hzTEc5aWMyVnlkbUYwYVc5dVgzSmxZMlZwY0hSek9sdGRMR052Ym5S"
    "eWIyeHNaWEpmYjJKelpYSjJZWFJwYjI1ek9sdGRMR3h2WTJGc1gzUnZiMnhmY21WalpXbHdkSE02VzEwc1lXTjBhVzl1Y3pw"
    "YlhTeGpiR1ZoYm5Wd1gyVnljbTl5Y3pwYlhTd0tJQ0JtYVc1aGJGOWhZbk5sYm1ObE9tNTFiR3dzYjNkdVpXUmZZMmhwYkdS"
    "ZlpYaHBkSE02Ym5Wc2JDeGpiMjUwY205c2JHVnlYMlY0YVhRNmJuVnNiQ3dLSUNCbWFYSnpkRjl2WW5ObGNuWmhkR2x2Ymw5"
    "MGIxOW1hVzVoYkY5M1lXeHNYMjF6T201MWJHd3NiR0YwWTJoZmRHOWZZV0p6Wlc1alpWOXRjenB1ZFd4c0NuMDdDbU52Ym5O"
    "MElITnhQWE05UGlJbklpdFRkSEpwYm1jb2N5a3VjbVZ3YkdGalpTZ3ZKeTluTENJblhGd25KeUlwS3lJbklqc0tZMjl1YzNR"
    "Z2NtVjBZV2x1UFNncFBUNXpkRzl5WlNoUFFsTkxSVmtzY21WamIzSmtLVHNLWTI5dWMzUWdZMnhsWVc1MWNFVnljbTl5UFd4"
    "aFltVnNQVDU3Q2lBZ2FXWW9JWEpsWTI5eVpDNWpiR1ZoYm5Wd1gyVnljbTl5Y3k1cGJtTnNkV1JsY3loc1lXSmxiQ2twY21W"
    "amIzSmtMbU5zWldGdWRYQmZaWEp5YjNKekxuQjFjMmdvYkdGaVpXd3BPd29nSUhKbGRHRnBiaWdwT3dwOU93cGpiMjV6ZENC"
    "c2IyTmhiRDFoYzNsdVl5aHpiM1Z5WTJVc1lYSm5LVDArZXdvZ0lHTnZibk4wSUdOdFpEMGljSGwwYUc5dU15QXRZeUFpSzNO"
    "eEtITnZkWEpqWlNrcktHRnlaejA5UFhWdVpHVm1hVzVsWkQ4aUlqb2lJQ0lyYzNFb1NsTlBUaTV6ZEhKcGJtZHBabmtvWVhK"
    "bktTa3BPd29nSUdOdmJuTjBJSEk5WVhkaGFYUWdkRzl2YkhNdVpYaGxZMTlqYjIxdFlXNWtLSHRqYldRc2QyOXlhMlJwY2pw"
    "dmQyNHVkMjl5YTNOd1lXTmxMQW9nSUNBZ2VXbGxiR1JmZEdsdFpWOXRjem94TURBd01DeHRZWGhmYjNWMGNIVjBYM1J2YTJW"
    "dWN6b3lNREF3ZlNrN0NpQWdjbVZqYjNKa0xteHZZMkZzWDNSdmIyeGZjbVZqWldsd2RITXVjSFZ6YUNoN0NpQWdJQ0JzWVdK"
    "bGJEcHpiM1Z5WTJVOVBUMURURTlEU3o4aVkyeHZZMnNpT25OdmRYSmpaVDA5UFZOVVQxQS9Jbk4wYjNBaU9uTnZkWEpqWlQw"
    "OVBWSkZRVVJDUVVOTFB5SnlaV0ZrWW1GamF5STZJblZ1YTI1dmQyNGlMQW9nSUNBZ1pYaHBkRHAwZVhCbGIyWWdjaTVsZUds"
    "MFgyTnZaR1U5UFQwaWJuVnRZbVZ5SWo5eUxtVjRhWFJmWTI5a1pUcHVkV3hzTEFvZ0lDQWdjMlZ6YzJsdmJsOXBaRHAwZVhC"
    "bGIyWWdjaTV6WlhOemFXOXVYMmxrUFQwOUltNTFiV0psY2lJL2NpNXpaWE56YVc5dVgybGtPbTUxYkd3S0lDQjlLVHR5WlhS"
    "aGFXNG9LVHNnTHk4Z1RuVnRaWEpwWXlCamIyeHNaV04wYjNJZ2NtVmpaV2x3ZENCQ1JVWlBVa1VnWTI5dGNHRnlhWE52Ym5N"
    "dUNpQWdhV1lvY2k1elpYTnphVzl1WDJsa0lUMDlkVzVrWldacGJtVmtLU0I3Q2lBZ0lDQmpiR1ZoYm5Wd1JYSnliM0lvSW05"
    "M2JtVmtYMnh2WTJGc1gyTnZiVzFoYm1SZmRXNXFiMmx1WldRaUtUdDBhSEp2ZHlCdVpYY2dSWEp5YjNJb0lteHZZMkZzWDNC"
    "bGJtUnBibWNpS1RzS0lDQjlDaUFnYVdZb2NpNWxlR2wwWDJOdlpHVWhQVDB3S1hSb2NtOTNJRzVsZHlCRmNuSnZjaWdpYkc5"
    "allXeGZabUZwYkdWa0lpazdDaUFnY21WMGRYSnVJRXBUVDA0dWNHRnljMlVvY2k1dmRYUndkWFFwT3dwOU93cG1kVzVqZEds"
    "dmJpQm1hVzVwZEdWUFluTmxjblpoZEdsdmJpaDJZV3gxWlNrZ2V3b2dJR052Ym5OMElHVjRZV04wUFNoMkxHdGxlWE1wUFQ1"
    "MklUMDliblZzYkNZbWRIbHdaVzltSUhZOVBUMGliMkpxWldOMElpWW1JVUZ5Y21GNUxtbHpRWEp5WVhrb2Rpa21KZ29nSUNB"
    "Z1QySnFaV04wTG10bGVYTW9kaWt1YkdWdVozUm9QVDA5YTJWNWN5NXNaVzVuZEdnbUptdGxlWE11WlhabGNua29hejArVDJK"
    "cVpXTjBMbWhoYzA5M2JpaDJMR3NwS1RzS0lDQnBaaWdoWlhoaFkzUW9kbUZzZFdVc1d5SnZZbk5sY25abFpDSXNJbVJwWVdk"
    "dWIzTjBhV01pWFNsOGZIUjVjR1Z2WmlCMllXeDFaUzV2WW5ObGNuWmxaQ0U5UFNKaWIyOXNaV0Z1SWlseVpYUjFjbTRnYm5W"
    "c2JEc0tJQ0JqYjI1emRDQmtQWFpoYkhWbExtUnBZV2R1YjNOMGFXTTdDaUFnYVdZb1pEMDlQVzUxYkd3cGNtVjBkWEp1SUh0"
    "dlluTmxjblpsWkRwMllXeDFaUzV2WW5ObGNuWmxaQ3hrYVdGbmJtOXpkR2xqT201MWJHeDlPd29nSUdsbUtDRjJZV3gxWlM1"
    "dlluTmxjblpsWkh4OElXVjRZV04wS0dRc1d5SnphWFJsSWl3aVpYaGpaWEIwYVc5dVgyTnNZWE56SWl3aWIzZHVYMloxYm1O"
    "MGFXOXVJaXdpYjNkdVgyeHBibVVpWFNsOGZBb2dJQ0FnSUNGYkltaGhibVJzWlhJaUxDSnpaWEoyWlhJaUxDSnRZV2x1SWww"
    "dWFXNWpiSFZrWlhNb1pDNXphWFJsS1h4OENpQWdJQ0FnSVZzaVFYUjBjbWxpZFhSbFJYSnliM0lpTENKVWVYQmxSWEp5YjNJ"
    "aUxDSldZV3gxWlVWeWNtOXlJaXdpUzJWNVJYSnliM0lpTENKUFUwVnljbTl5SWl3aVFuSnZhMlZ1VUdsd1pVVnljbTl5SWl3"
    "S0lDQWdJQ0FnSUNKRGIyNXVaV04wYVc5dVVtVnpaWFJGY25KdmNpSXNJbFJwYldWdmRYUkZjbkp2Y2lJc0ltOTBhR1Z5SWww"
    "dWFXNWpiSFZrWlhNb1pDNWxlR05sY0hScGIyNWZZMnhoYzNNcEtYSmxkSFZ5YmlCdWRXeHNPd29nSUdOdmJuTjBJR1oxYm1O"
    "MGFXOXVjejFiSWtobFlXUmxjbEpsWVdSbGNpNXlaV0ZrYkdsdVpTSXNJa1JsYlc5VFpYSjJaWEl1Y0hKdlkyVnpjMTl5WlhG"
    "MVpYTjBJaXdpUkdWdGJ5NWlaV2RwYmlJc0lrUmxiVzh1WTJGc2JHSmhZMnNpTEFvZ0lDQWdJa1JsYlc4dWFXNTJiMnRsSWl3"
    "aVNHRnVaR3hsY2k1b1lXNWtiR1ZmYjI1bFgzSmxjWFZsYzNRaUxDSklZVzVrYkdWeUxuTmxibVJmWlhKeWIzSWlMQ0pJWVc1"
    "a2JHVnlMbWRsZENJc0lraGhibVJzWlhJdWNtVndiSGtpTENKdFlXbHVJbDA3Q2lBZ2FXWW9JU2dvWkM1dmQyNWZablZ1WTNS"
    "cGIyNDlQVDF1ZFd4c0ppWmtMbTkzYmw5c2FXNWxQVDA5Ym5Wc2JDbDhmQW9nSUNBZ0lDQWdLR1oxYm1OMGFXOXVjeTVwYm1O"
    "c2RXUmxjeWhrTG05M2JsOW1kVzVqZEdsdmJpa21KazUxYldKbGNpNXBjMU5oWm1WSmJuUmxaMlZ5S0dRdWIzZHVYMnhwYm1V"
    "cEppWUtJQ0FnSUNBZ0lDQmtMbTkzYmw5c2FXNWxQajB4Smlaa0xtOTNibDlzYVc1bFBEMHhNREkwS1NrcGNtVjBkWEp1SUc1"
    "MWJHdzdDaUFnY21WMGRYSnVJSHR2WW5ObGNuWmxaRHAwY25WbExHUnBZV2R1YjNOMGFXTTZlM05wZEdVNlpDNXphWFJsTEdW"
    "NFkyVndkR2x2Ymw5amJHRnpjenBrTG1WNFkyVndkR2x2Ymw5amJHRnpjeXdLSUNBZ0lHOTNibDltZFc1amRHbHZianBrTG05"
    "M2JsOW1kVzVqZEdsdmJpeHZkMjVmYkdsdVpUcGtMbTkzYmw5c2FXNWxmWDA3Q24wS1puVnVZM1JwYjI0Z1pHbGhaMjV2YzNS"
    "cFl5aDJZV3gxWlN4d2NtOXFaV04wYVc5dUtTQjdDaUFnWTI5dWMzUWdjSEp2YW1WamRHVmtQV1pwYm1sMFpVOWljMlZ5ZG1G"
    "MGFXOXVLSFpoYkhWbEtUc0tJQ0JwWmlod2NtOXFaV04wYVc5dVBUMDlJbWx1ZG1Gc2FXUWlmSHdoV3lKMllXeHBaQ0lzSW5W"
    "dVlYWmhhV3hoWW14bElsMHVhVzVqYkhWa1pYTW9jSEp2YW1WamRHbHZiaWw4ZkFvZ0lDQWdJQ2h3Y205cVpXTjBhVzl1UFQw"
    "OUluWmhiR2xrSWlZbWNISnZhbVZqZEdWa1BUMDliblZzYkNrcElIc0tJQ0FnSUdsbUtDRnlaV052Y21RdVpHbGhaMjV2YzNS"
    "cFkxOWxjbkp2Y25NdWFXNWpiSFZrWlhNb0ltOWljMlZ5ZG1WeVgzQnliMnBsWTNScGIyNWZhVzUyWVd4cFpDSXBLUW9nSUNB"
    "Z0lDQnlaV052Y21RdVpHbGhaMjV2YzNScFkxOWxjbkp2Y25NdWNIVnphQ2dpYjJKelpYSjJaWEpmY0hKdmFtVmpkR2x2Ymw5"
    "cGJuWmhiR2xrSWlrN0NpQWdmUW9nSUdsbUtIQnliMnBsWTNSbFpDRTlQVzUxYkd3bUpuSmxZMjl5WkM1MWJtVjRjR1ZqZEdW"
    "a1gyWmhhV3gxY21WZmIySnpaWEoyWVhScGIyNDlQVDF1ZFd4c0tRb2dJQ0FnY21WamIzSmtMblZ1Wlhod1pXTjBaV1JmWm1G"
    "cGJIVnlaVjl2WW5ObGNuWmhkR2x2Ymoxd2NtOXFaV04wWldRN0NpQWdjbVYwWVdsdUtDazdJQzh2SUU1dklISmhkeUJrYVdG"
    "bmJtOXpkR2xqSUdaaGJHeGlZV05ySUdGdVpDQnVieUJtYVhKemRDMW1ZV2xzZFhKbElHOXlJRzkxZEdOdmJXVWdiWFYwWVhS"
    "cGIyNHVDbjBLWTI5dWMzUWdibk05WXowK1FtbG5TVzUwS0dNdWJXOXViM1J2Ym1salgyNXpLVHNLWTI5dWMzUWdaMlYwUTJ4"
    "dlkyczlLQ2s5UG14dlkyRnNLRU5NVDBOTEtUc0tiR1YwSUdOdmJuUnliMnhzWlhKS2IybHVaV1E5Wm1Gc2MyVTdDbXhsZENC"
    "amIyNTBjbTlzYkdWeVVHOXNiRUYyWVdsc1lXSnNaVDEwY25WbE93cHNaWFFnWTI5dWRISnZiR3hsY2tKMVptWmxjajBpSWpz"
    "S2JHVjBJR05zWldGdWRYQlRkR0Z5ZEdWa1BXWmhiSE5sT3dwc1pYUWdiR0Z6ZEZOdVlYQnphRzkwUm14aFozTTliblZzYkRz"
    "S1puVnVZM1JwYjI0Z2JHRjBZMmdvYkdGaVpXd3NjbVZqWldsMlpXUlhZV3hzS1NCN0NpQWdhV1lvY21WamIzSmtMbVpwY25O"
    "MFgyWmhhV3gxY21VOVBUMXVkV3hzS1NCN0NpQWdJQ0J5WldOdmNtUXVabWx5YzNSZlptRnBiSFZ5WlQxc1lXSmxiRHNLSUNB"
    "Z0lISmxZMjl5WkM1bWFYSnpkRjl2WW5ObGNuWmhkR2x2Ymw5M1lXeHNYMjF6UFhKbFkyVnBkbVZrVjJGc2JEc0tJQ0FnSUhK"
    "bGRHRnBiaWdwT3lBdkx5QlRlVzVqYUhKdmJtOTFjeUJtYVhKemRDMW1ZV2xzZFhKbEwzZGhiR3dnYkdGMFkyZ2dRa1ZHVDFK"
    "RklHRnVlU0J1WlhjZ1lYZGhhWFF2YjNWMGNIVjBMZ29nSUgwS2ZRcG1kVzVqZEdsdmJpQnZZbk5sY25abFEyOXVkSEp2Ykd4"
    "bGNpaHlMSEpsWTJWcGRtVmtWMkZzYkNrZ2V3b2dJR052Ym5OMElHOWljMlZ5ZG1GMGFXOXVQWHR5WldObGFYWmxaRjkzWVd4"
    "c1gyMXpPbkpsWTJWcGRtVmtWMkZzYkN3S0lDQWdJR1Y0YVhRNmRIbHdaVzltSUhJdVpYaHBkRjlqYjJSbFBUMDlJbTUxYldK"
    "bGNpSS9jaTVsZUdsMFgyTnZaR1U2Ym5Wc2JDd0tJQ0FnSUdobGJIQmxjbDlqYjIxd2JHVjBaV1E2Wm1Gc2MyVXNhR1ZzY0dW"
    "eVgyVjRhWFE2Ym5Wc2JDeG1hWGgwZFhKbFgyWnBibWx6YUdWa09tWmhiSE5sZlRzS0lDQXZMeUJEYjIxd2JHVjBaU0J1ZFcx"
    "bGNtbGpJSEpsYzNWc2RDQndjbTlxWldOMGFXOXVJR2x6SUhKbGRHRnBibVZrSUVKRlJrOVNSU0JqYjIxd1lYSnBjMjl1Y3k0"
    "S0lDQnlaV052Y21RdVkyOXVkSEp2Ykd4bGNsOXZZbk5sY25aaGRHbHZibk11Y0hWemFDaHZZbk5sY25aaGRHbHZiaWs3Y21W"
    "MFlXbHVLQ2s3Q2lBZ2FXWW9kSGx3Wlc5bUlISXVaWGhwZEY5amIyUmxQVDA5SW01MWJXSmxjaUlwSUhzS0lDQWdJR052Ym5S"
    "eWIyeHNaWEpLYjJsdVpXUTlkSEoxWlR0eVpXTnZjbVF1WTI5dWRISnZiR3hsY2w5bGVHbDBQWEl1WlhocGRGOWpiMlJsTzNK"
    "bGRHRnBiaWdwT3dvZ0lIMEtJQ0JqYjI1MGNtOXNiR1Z5UW5WbVptVnlLejEwZVhCbGIyWWdjaTV2ZFhSd2RYUTlQVDBpYzNS"
    "eWFXNW5Jajl5TG05MWRIQjFkRG9pSWpzS0lDQnBaaWhqYjI1MGNtOXNiR1Z5UW5WbVptVnlMbXhsYm1kMGFENHhOak00TkNr"
    "Z2V3b2dJQ0FnYkdGMFkyZ29JbU52Ym5SeWIyeHNaWEpmYjJKelpYSjJZWFJwYjI1ZmFXNTJZV3hwWkNJc2NtVmpaV2wyWldS"
    "WFlXeHNLVHRqYjI1MGNtOXNiR1Z5UW5WbVptVnlQU0lpTzNKbGRIVnlianNLSUNCOUNpQWdiR1YwSUdOMWREc0tJQ0IzYUds"
    "c1pTZ29ZM1YwUFdOdmJuUnliMnhzWlhKQ2RXWm1aWEl1YVc1a1pYaFBaaWdpWEc0aUtTaytQVEFwSUhzS0lDQWdJR052Ym5O"
    "MElHeHBibVU5WTI5dWRISnZiR3hsY2tKMVptWmxjaTV6YkdsalpTZ3dMR04xZENrN1kyOXVkSEp2Ykd4bGNrSjFabVpsY2ox"
    "amIyNTBjbTlzYkdWeVFuVm1abVZ5TG5Oc2FXTmxLR04xZENzeEtUc0tJQ0FnSUdsbUtDRnNhVzVsTG5SeWFXMG9LU2xqYjI1"
    "MGFXNTFaVHNLSUNBZ0lHeGxkQ0JsZG1WdWREc0tJQ0FnSUhSeWVYdGxkbVZ1ZEQxS1UwOU9MbkJoY25ObEtHeHBibVVwTzMx"
    "allYUmphSHNLSUNBZ0lDQWdiR0YwWTJnb0ltTnZiblJ5YjJ4c1pYSmZiMkp6WlhKMllYUnBiMjVmYVc1MllXeHBaQ0lzY21W"
    "alpXbDJaV1JYWVd4c0tUdGpiMjUwYVc1MVpUc0tJQ0FnSUgwS0lDQWdJR2xtS0U5aWFtVmpkQzVvWVhOUGQyNG9aWFpsYm5R"
    "c0luVnVaWGh3WldOMFpXUmZabUZwYkhWeVpWOXZZbk5sY25aaGRHbHZiaUlwS1FvZ0lDQWdJQ0JrYVdGbmJtOXpkR2xqS0dW"
    "MlpXNTBMblZ1Wlhod1pXTjBaV1JmWm1GcGJIVnlaVjl2WW5ObGNuWmhkR2x2Yml4bGRtVnVkQzUxYm1WNGNHVmpkR1ZrWDI5"
    "aWMyVnlkbUYwYVc5dVgzQnliMnBsWTNScGIyNHBPd29nSUNBZ2FXWW9aWFpsYm5RdVptbDRkSFZ5WlY5eVpXRmtlVDA5UFhS"
    "eWRXVXBJSHNLSUNBZ0lDQWdhV1lvWlhabGJuUXVaM1ZoY21SZmNHbGtJVDA5YjNkdUxtZDFZWEprWDNCcFpIeDhaWFpsYm5R"
    "dWMyVnlkbVZ5WDNCcFpDRTlQVzkzYmk1elpYSjJaWEpmY0dsa2ZId0tJQ0FnSUNBZ0lDQWdaWFpsYm5RdWJHRmlJVDA5YjNk"
    "dUxteGhZbng4SVU1MWJXSmxjaTVwYzFOaFptVkpiblJsWjJWeUtHVjJaVzUwTG1obGJIQmxjbDl3YVdRcGZIeGxkbVZ1ZEM1"
    "b1pXeHdaWEpmY0dsa1BEMHdmSHdLSUNBZ0lDQWdJQ0FnVzI5M2JpNW5kV0Z5WkY5d2FXUXNiM2R1TG5ObGNuWmxjbDl3YVdR"
    "c2IzZHVMbUp5YjNkelpYSmZjR2xrWFM1cGJtTnNkV1JsY3lobGRtVnVkQzVvWld4d1pYSmZjR2xrS1h4OENpQWdJQ0FnSUNB"
    "Z0lDaHZkMjR1YUdWc2NHVnlYM0JwWkNFOVBXNTFiR3dtSm05M2JpNW9aV3h3WlhKZmNHbGtJVDA5WlhabGJuUXVhR1ZzY0dW"
    "eVgzQnBaQ2twQ2lBZ0lDQWdJQ0FnYkdGMFkyZ29JbVpwZUhSMWNtVmZjbVZoWkhsZmRXNWpiMjVtYVhKdFpXUWlMSEpsWTJW"
    "cGRtVmtWMkZzYkNrN0NpQWdJQ0FnSUdWc2MyVWdld29nSUNBZ0lDQWdJRzkzYmk1b1pXeHdaWEpmY0dsa1BXVjJaVzUwTG1o"
    "bGJIQmxjbDl3YVdRN2IzZHVMbVpwZUhSMWNtVmZjbVZoWkhrOWRISjFaVHR2ZDI0dWJHbHpkR1Z1WlhKZmNHbGtYM0J5YjI5"
    "bVBYUnlkV1U3Q2lBZ0lDQWdJQ0FnYzNSdmNtVW9JbVF3TVY5cGJXMWxaR2xoZEdWZmIzZHVaV1JmYUdGdVpHeGxjeUlzYjNk"
    "dUtUc0tJQ0FnSUNBZ2ZRb2dJQ0FnZlFvZ0lDQWdhV1lvWlhabGJuUXVhR1ZzY0dWeVgyTnZiWEJzWlhSbFpEMDlQWFJ5ZFdV"
    "cElIc0tJQ0FnSUNBZ2IySnpaWEoyWVhScGIyNHVhR1ZzY0dWeVgyTnZiWEJzWlhSbFpEMTBjblZsT3dvZ0lDQWdJQ0J2WW5O"
    "bGNuWmhkR2x2Ymk1b1pXeHdaWEpmWlhocGREMTBlWEJsYjJZZ1pYWmxiblF1WlhocGREMDlQU0p1ZFcxaVpYSWlQMlYyWlc1"
    "MExtVjRhWFE2Ym5Wc2JEc0tJQ0FnSUNBZ2NtVjBZV2x1S0NrN0NpQWdJQ0FnSUdsbUtHVjJaVzUwTG1WNGFYUWhQVDB3S1d4"
    "aGRHTm9LQ0pvWld4d1pYSmZabUZwYkdWa0lpeHlaV05sYVhabFpGZGhiR3dwT3dvZ0lDQWdJQ0JsYkhObElHbG1LQ0ZqYkdW"
    "aGJuVndVM1JoY25SbFpDWW1jbVZqYjNKa0xuQnliM1JsWTNSbFpGOWhablJsY2w5d1lXZGxJVDA5ZEhKMVpTWW1DaUFnSUNB"
    "Z0lDQWdJQ0FnSUNBZ0lTaHZkMjR1Y0doaGMyVTlQVDBpWW5KdmQzTmxjbDlrWldOcGMybHZiaUltSmdvZ0lDQWdJQ0FnSUNB"
    "Z0lDQWdJQ0FnYjNkdUxtNWxlSFJmWkdWamFYTnBiMjQvTG10cGJtUTlQVDBpY0hKdmRHVmpkR1ZrWDJGbWRHVnlJaWtwQ2lB"
    "Z0lDQWdJQ0FnYkdGMFkyZ29JbWhsYkhCbGNsOWpiMjF3YkdWMFpXUmZZbVZtYjNKbFgyRndjRjlqYUdWamEzQnZhVzUwSWl4"
    "eVpXTmxhWFpsWkZkaGJHd3BPd29nSUNBZ2ZRb2dJQ0FnYVdZb1pYWmxiblF1Wm1sNGRIVnlaVjltYVc1cGMyaGxaRDA5UFhS"
    "eWRXVXBJSHNLSUNBZ0lDQWdiMkp6WlhKMllYUnBiMjR1Wm1sNGRIVnlaVjltYVc1cGMyaGxaRDEwY25WbE8zSmxkR0ZwYmln"
    "cE93b2dJQ0FnSUNCcFppZ2hZMnhsWVc1MWNGTjBZWEowWldRcGJHRjBZMmdvSW1OdmJuUnliMnhzWlhKZlkyOXRjR3hsZEdW"
    "a1gySmxabTl5WlY5aGNIQmZZMmhsWTJ0d2IybHVkQ0lzY21WalpXbDJaV1JYWVd4c0tUc0tJQ0FnSUgwS0lDQjlDaUFnYVdZ"
    "b1kyOXVkSEp2Ykd4bGNrcHZhVzVsWkNZbUlXTnNaV0Z1ZFhCVGRHRnlkR1ZrS1FvZ0lDQWdiR0YwWTJnb0ltTnZiblJ5YjJ4"
    "c1pYSmZZMjl0Y0d4bGRHVmtYMkpsWm05eVpWOWhjSEJmWTJobFkydHdiMmx1ZENJc2NtVmpaV2wyWldSWFlXeHNLVHNLZlFw"
    "aGMzbHVZeUJtZFc1amRHbHZiaUJ3YjJ4c1EyOXVkSEp2Ykd4bGNpZ3BJSHNLSUNCcFppaGpiMjUwY205c2JHVnlTbTlwYm1W"
    "a2ZId2hZMjl1ZEhKdmJHeGxjbEJ2Ykd4QmRtRnBiR0ZpYkdVcGNtVjBkWEp1T3dvZ0lIUnllU0I3Q2lBZ0lDQmpiMjV6ZENC"
    "eVBXRjNZV2wwSUhSdmIyeHpMbmR5YVhSbFgzTjBaR2x1S0h0elpYTnphVzl1WDJsa09tOTNiaTVsZUdWalgzTmxjM05wYjI0"
    "c0NpQWdJQ0FnSUdOb1lYSnpPaUlpTEhscFpXeGtYM1JwYldWZmJYTTZOVEF3TUN4dFlYaGZiM1YwY0hWMFgzUnZhMlZ1Y3pv"
    "eU1EQXdmU2s3Q2lBZ0lDQmpiMjV6ZENCeVpXTmxhWFpsWkZkaGJHdzlSR0YwWlM1dWIzY29LVHNLSUNBZ0lHOWljMlZ5ZG1W"
    "RGIyNTBjbTlzYkdWeUtISXNjbVZqWldsMlpXUlhZV3hzS1RzS0lDQjlJR05oZEdOb0lIc0tJQ0FnSUdOdmJuUnliMnhzWlhK"
    "UWIyeHNRWFpoYVd4aFlteGxQV1poYkhObE93b2dJQ0FnYkdGMFkyZ29JbU52Ym5SeWIyeHNaWEpmYjJKelpYSjJZWFJwYjI1"
    "ZmRXNWhkbUZwYkdGaWJHVWlMRVJoZEdVdWJtOTNLQ2twT3dvZ0lDQWdhV1lvWTJ4bFlXNTFjRk4wWVhKMFpXUXBZMnhsWVc1"
    "MWNFVnljbTl5S0NKamIyNTBjbTlzYkdWeVgycHZhVzVmZFc1amIyNW1hWEp0WldRaUtUc0tJQ0I5Q24wS1lYTjVibU1nWm5W"
    "dVkzUnBiMjRnZEdsdFpXUkVjbWwyWlhJb2JHRmlaV3dzWVd4c2IyTmhkR2x2Yml4dmNHVnlZWFJwYjI0c2NISnZhbVZqZENr"
    "Z2V3b2dJR3hsZENCemRHRnlkRDF1ZFd4c0xHVnVaRDF1ZFd4c0xISmxjM1ZzZEQxdWRXeHNMSE4wWVhSbFBTSjFibXR1YjNk"
    "dUlqc0tJQ0IwY25sN2MzUmhjblE5WVhkaGFYUWdaMlYwUTJ4dlkyc29LVHQ5WTJGMFkyaDdZMnhsWVc1MWNFVnljbTl5S0NK"
    "amJHOWphMTkxYm1GMllXbHNZV0pzWlNJcE8zMEtJQ0JqYjI1emRDQmhZM1JwYjI0OWUyeGhZbVZzTEhOMFlYSjBYMk5zYjJO"
    "ck9uTjBZWEowTEdWdVpGOWpiRzlqYXpwdWRXeHNMR0ZzYkc5M1pXUmZiWE02WVd4c2IyTmhkR2x2Yml3S0lDQWdJSEpsYzNW"
    "c2RGOXpkR0YwWlRvaWRXNXJibTkzYmlJc1pXeGhjSE5sWkY5dGN6cHVkV3hzTEc5MlpYSmZZblZrWjJWME9tNTFiR3g5T3dv"
    "Z0lISmxZMjl5WkM1aFkzUnBiMjV6TG5CMWMyZ29ZV04wYVc5dUtUdHlaWFJoYVc0b0tUc2dMeThnVTNSaGNuUWdjbVYwWVds"
    "dVpXUWdRa1ZHVDFKRklHVnVkR1Z5YVc1bklFUnlhWFpsY2lCallXeHNMZ29nSUhSeWVTQjdDaUFnSUNCeVpYTjFiSFE5WVhk"
    "aGFYUWdiM0JsY21GMGFXOXVLQ2s3Q2lBZ0lDQnpkR0YwWlQxeVpYTjFiSFEvTG1selJYSnliM0k5UFQxMGNuVmxQeUp5Wlda"
    "MWMyVmtJam9pY21WMGRYSnVaV1FpT3dvZ0lIMGdZMkYwWTJnZ2UzTjBZWFJsUFNKbGVHTmxjSFJwYjI0aU8zMEtJQ0IwY25s"
    "N1pXNWtQV0YzWVdsMElHZGxkRU5zYjJOcktDazdmV05oZEdOb2UyTnNaV0Z1ZFhCRmNuSnZjaWdpWTJ4dlkydGZkVzVoZG1G"
    "cGJHRmliR1VpS1R0OUNpQWdZV04wYVc5dUxtVnVaRjlqYkc5amF6MWxibVE3WVdOMGFXOXVMbkpsYzNWc2RGOXpkR0YwWlQx"
    "emRHRjBaVHNLSUNCeVpYUmhhVzRvS1RzZ0x5OGdSVzVrTDNKbGMzVnNkQ0J5WlhSaGFXNWxaQ0JDUlVaUFVrVWdZblZrWjJW"
    "MElHTnZiWEJoY21semIyNHVDaUFnYVdZb2MzUmhjblFtSm1WdVpDa2dld29nSUNBZ1kyOXVjM1FnWkQxdWN5aGxibVFwTFc1"
    "ektITjBZWEowS1RzS0lDQWdJR2xtS0dRK1BUQnVLU0I3Q2lBZ0lDQWdJR0ZqZEdsdmJpNWxiR0Z3YzJWa1gyMXpQVTUxYldK"
    "bGNpaGtMekV3TURBd01EQnVLVHNLSUNBZ0lDQWdZV04wYVc5dUxtOTJaWEpmWW5Wa1oyVjBQV0ZqZEdsdmJpNWxiR0Z3YzJW"
    "a1gyMXpQbUZzYkc5allYUnBiMjQ3Q2lBZ0lDQjlJR1ZzYzJVZ1kyeGxZVzUxY0VWeWNtOXlLQ0pqYkc5amExOXBiblpoYkds"
    "a0lpazdDaUFnZlFvZ0lHbG1LR0ZqZEdsdmJpNXZkbVZ5WDJKMVpHZGxkQ2xqYkdWaGJuVndSWEp5YjNJb0ltUnlhWFpsY2w5"
    "dmNHVnlZWFJwYjI1ZmIzWmxjbDlpZFdSblpYUWlLVHNLSUNCcFppaHpkR0YwWlNFOVBTSnlaWFIxY201bFpDSXBZMnhsWVc1"
    "MWNFVnljbTl5S0NKa2NtbDJaWEpmYjNCbGNtRjBhVzl1WDNWdVkyOXVabWx5YldWa0lpazdDaUFnYVdZb2NISnZhbVZqZENs"
    "d2NtOXFaV04wS0hKbGMzVnNkQ3h6ZEdGMFpTazdDaUFnY21WMFlXbHVLQ2s3Q24wS1lYTjVibU1nWm5WdVkzUnBiMjRnWTJ4"
    "bFlXNTFjQ2dwSUhzS0lDQnpkRzl5WlNnaVpEQXhYMlp5WlhOb1gzQmhjM04zYjNKa1gybHVjSFYwSWl4dWRXeHNLVHNnTHk4"
    "Z1EyeGxZWElnWW1WbWIzSmxJR0Z1ZVNCamJHVmhiblZ3SUdGM1lXbDBMZ29nSUdsbUtHTnNaV0Z1ZFhCVGRHRnlkR1ZrS1hK"
    "bGRIVnlianNLSUNCamJHVmhiblZ3VTNSaGNuUmxaRDEwY25WbE8zSmxZMjl5WkM1amJHVmhiblZ3WDNOMFlYSjBaV1E5ZEhK"
    "MVpUdHlaWFJoYVc0b0tUc0tJQ0IwY25rZ2V3b2dJQ0FnWTI5dWMzUWdjajFoZDJGcGRDQnNiMk5oYkNoVFZFOVFMSHNLSUNB"
    "Z0lDQWdiR0ZpT205M2JpNXNZV0lzWlhabGJuUmZiM1YwT205M2JpNWxkbVZ1ZEY5dmRYUXNDaUFnSUNBZ0lHWnBjbk4wWDJa"
    "aGFXeDFjbVU2Y21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21Vc0NpQWdJQ0FnSUdacGNuTjBYMjlpYzJWeWRtRjBhVzl1WDNk"
    "aGJHeGZiWE02Y21WamIzSmtMbVpwY25OMFgyOWljMlZ5ZG1GMGFXOXVYM2RoYkd4ZmJYTUtJQ0FnSUgwcE93b2dJQ0FnY21W"
    "amIzSmtMbVpwY25OMFgyTnNiMk5yUFhJdVkyeHZZMnM3Y21WamIzSmtMbk4wYjNCZmMzUmhkR1U5Y2k1dFlYSnJaWEpmYzNS"
    "aGRHVTdjbVYwWVdsdUtDazdDaUFnSUNCcFppaHlMbVYyWlc1MFgzTjBZWFJsSVQwOUluZHlhWFIwWlc0aUtXTnNaV0Z1ZFhC"
    "RmNuSnZjaWdpYjJKelpYSjJZWFJwYjI1ZmJXVjBZV1JoZEdGZmQzSnBkR1ZmZFc1amIyNW1hWEp0WldRaUtUc0tJQ0FnSUds"
    "bUtISXViV0Z5YTJWeVgzTjBZWFJsUFQwOUluTjBiM0JmZFc1amIyNW1hWEp0WldRaUtXTnNaV0Z1ZFhCRmNuSnZjaWdpYzNS"
    "dmNGOTFibU52Ym1acGNtMWxaQ0lwT3dvZ0lDQWdhV1lvY2k1dFlYSnJaWEpmYzNSaGRHVTlQVDBpYjNkdVpYSnphR2x3WDNW"
    "dWEyNXZkMjRpS1dOc1pXRnVkWEJGY25KdmNpZ2liM2R1WlhKemFHbHdYM1Z1YTI1dmQyNGlLVHNLSUNCOUlHTmhkR05vSUh0"
    "amJHVmhiblZ3UlhKeWIzSW9Jbk4wYjNCZmIzSmZZMnh2WTJ0ZmNtVmpiM0prWDNWdVlYWmhhV3hoWW14bElpazdmUW9nSUM4"
    "dklFNXZJRzkxZEhOMFlXNWthVzVuSUVSeWFYWmxjaUJqWVd4c0lHVjRhWE4wY3lCb1pYSmxPaUJoYkd3Z2NHRm5aU0JqWVd4"
    "c2N5QmhZbTkyWlNCM1pYSmxJR0YzWVdsMFpXUXVDaUFnTHk4Z1QzSnBaMmx1WVd3Z2MzUnZjQ0J3Y205MGIyTnZiQ0JoYkhK"
    "bFlXUjVJR05oZFhObGN5QjBhR1VnWTI5dWRISnZiR3hsY2lkeklHOTNibVZrTFdOb2FXeGtJR1pwYm1Gc2JIa3VDaUFnWVhk"
    "aGFYUWdkR2x0WldSRWNtbDJaWElvSW10cGJHeGZZWEJ3SWl3ek1EQXdNQ3dLSUNBZ0lDZ3BQVDUwYjI5c2N5NXRZM0JmWDJO"
    "MVlWOWtjbWwyWlhKZlgydHBiR3hmWVhCd0tIdHdhV1E2YjNkdUxtSnliM2R6WlhKZmNHbGtmU2twT3dvZ0lHRjNZV2wwSUhS"
    "cGJXVmtSSEpwZG1WeUtDSmxibVJmYzJWemMybHZiaUlzTVRVd01EQXNDaUFnSUNBb0tUMCtkRzl2YkhNdWJXTndYMTlqZFdG"
    "ZlpISnBkbVZ5WDE5bGJtUmZjMlZ6YzJsdmJpaDdjMlZ6YzJsdmJqcHZkMjR1YzJWemMybHZibjBwTENoeUxITjBZWFJsS1Qw"
    "K2V3b2dJQ0FnSUNCeVpXTnZjbVF1YzJWemMybHZibDlsYm1SbFpEMXpkR0YwWlQwOVBTSnlaWFIxY201bFpDSW1KZ29nSUNB"
    "Z0lDQWdJSEkvTG5OMGNuVmpkSFZ5WldSRGIyNTBaVzUwUHk1aFkzUnBkbVU5UFQxbVlXeHpaU1ltY2k1emRISjFZM1IxY21W"
    "a1EyOXVkR1Z1ZEM1elpYTnphVzl1UFQwOWIzZHVMbk5sYzNOcGIyNDdDaUFnSUNBZ0lISmxkR0ZwYmlncE93b2dJQ0FnZlNr"
    "N0NpQWdZWGRoYVhRZ2RHbHRaV1JFY21sMlpYSW9JbXhwYzNSZmQybHVaRzkzY3lJc05UQXdNQ3dLSUNBZ0lDZ3BQVDUwYjI5"
    "c2N5NXRZM0JmWDJOMVlWOWtjbWwyWlhKZlgyeHBjM1JmZDJsdVpHOTNjeWg3Y0dsa09tOTNiaTVpY205M2MyVnlYM0JwWkgw"
    "cExDaHlMSE4wWVhSbEtUMCtld29nSUNBZ0lDQmpiMjV6ZENCM2FXNWtiM2R6UFhJL0xuTjBjblZqZEhWeVpXUkRiMjUwWlc1"
    "MFB5NTNhVzVrYjNkek93b2dJQ0FnSUNCeVpXTnZjbVF1ZDJsdVpHOTNYMk52ZFc1MFBYTjBZWFJsUFQwOUluSmxkSFZ5Ym1W"
    "a0lpWW1RWEp5WVhrdWFYTkJjbkpoZVNoM2FXNWtiM2R6S1Q5M2FXNWtiM2R6TG14bGJtZDBhRHB1ZFd4c093b2dJQ0FnSUNC"
    "eVpYUmhhVzRvS1RzS0lDQWdJSDBwT3dvZ0lHeGxkQ0J5WldGa1UzUmhjblE5Ym5Wc2JEc0tJQ0IwY25sN2NtVmhaRk4wWVhK"
    "MFBXRjNZV2wwSUdkbGRFTnNiMk5yS0NrN2ZXTmhkR05vZTJOc1pXRnVkWEJGY25KdmNpZ2lZMnh2WTJ0ZmRXNWhkbUZwYkdG"
    "aWJHVWlLVHQ5Q2lBZ1kyOXVjM1FnWVdOMGFXOXVQWHRzWVdKbGJEb2liM2R1WldSZmFtOXBibDl5WldGa1ltRmpheUlzYzNS"
    "aGNuUmZZMnh2WTJzNmNtVmhaRk4wWVhKMExHVnVaRjlqYkc5amF6cHVkV3hzTEFvZ0lDQWdZV3hzYjNkbFpGOXRjenB1ZFd4"
    "c0xHVnNZWEJ6WldSZmJYTTZiblZzYkN4dmRtVnlYMkoxWkdkbGREcHVkV3hzTEhKbGMzVnNkRjl6ZEdGMFpUb2lkVzVyYm05"
    "M2JpSjlPd29nSUhKbFkyOXlaQzVoWTNScGIyNXpMbkIxYzJnb1lXTjBhVzl1S1R0eVpYUmhhVzRvS1RzS0lDQnBaaWh5WldG"
    "a1UzUmhjblFtSm5KbFkyOXlaQzVtYVhKemRGOWpiRzlqYXlrZ2V3b2dJQ0FnWVdOMGFXOXVMbUZzYkc5M1pXUmZiWE05VFdG"
    "MGFDNXRZWGdvTUN3Mk1EQXdNQzFPZFcxaVpYSW9LRzV6S0hKbFlXUlRkR0Z5ZENrdGJuTW9jbVZqYjNKa0xtWnBjbk4wWDJO"
    "c2IyTnJLU2t2TVRBd01EQXdNRzRwS1RzS0lDQWdJSEpsZEdGcGJpZ3BPd29nSUgwS0lDQXZMeUJEYjI1MGFXNTFaU0JsYzNO"
    "bGJuUnBZV3dnYW05cGJpQmhablJsY2pZd2N5QnBaaUJzWVhSbE95QnVaWFpsY2lCamJHRnBiU0IwYUdGMElHUmxZV1JzYVc1"
    "bElHVnVabTl5WTJWa0xnb2dJQzh2SUVSdklHNXZkQ0J3YjJ4c0lHRnViM1JvWlhJZ1pYaGxZeUJ6WlhOemFXOXVJRzl5SUhO"
    "bGJtUWdZU0J0WVc1MVlXd2djSEp2WTJWemN5QnphV2R1WVd3dUNpQWdkMmhwYkdVb0lXTnZiblJ5YjJ4c1pYSktiMmx1WldR"
    "bUptTnZiblJ5YjJ4c1pYSlFiMnhzUVhaaGFXeGhZbXhsSmlaRVlYUmxMbTV2ZHlncFBHOTNiaTV6ZEdGeWRGOWxjRzlqYUY5"
    "dGN5czVNREF3TURBcElIc0tJQ0FnSUdGM1lXbDBJSEJ2Ykd4RGIyNTBjbTlzYkdWeUtDazdDaUFnSUNCcFppaHlaV052Y21R"
    "dVptbHljM1JmWTJ4dlkyc3BJSHNLSUNBZ0lDQWdkSEo1SUhzS0lDQWdJQ0FnSUNCamIyNXpkQ0JqUFdGM1lXbDBJR2RsZEVO"
    "c2IyTnJLQ2s3Y21WamIzSmtMbXhoYzNSZmFtOXBibDlqYkc5amF6MWpPM0psZEdGcGJpZ3BPd29nSUNBZ0lDQWdJR2xtS0c1"
    "ektHTXBMVzV6S0hKbFkyOXlaQzVtYVhKemRGOWpiRzlqYXlrK05qQXdNREF3TURBd01EQnVLUW9nSUNBZ0lDQWdJQ0FnWTJ4"
    "bFlXNTFjRVZ5Y205eUtDSmpiR1ZoYm5Wd1gySjFaR2RsZEY5bGVHTmxaV1JsWkNJcE93b2dJQ0FnSUNCOUlHTmhkR05vZTJO"
    "c1pXRnVkWEJGY25KdmNpZ2lZMnh2WTJ0ZmRXNWhkbUZwYkdGaWJHVWlLVHQ5Q2lBZ0lDQjlDaUFnZlFvZ0lHbG1LQ0ZqYjI1"
    "MGNtOXNiR1Z5U205cGJtVmtLV05zWldGdWRYQkZjbkp2Y2lnaVkyaHBiR1JmY21WaGNGOXBibU52YlhCc1pYUmxJaWs3Q2lB"
    "Z2RISjVJSHNLSUNBZ0lHTnZibk4wSUhJOVlYZGhhWFFnYkc5allXd29Va1ZCUkVKQlEwc3Nld29nSUNBZ0lDQnZkMjVsWkY5"
    "d2FXUnpPbHR2ZDI0dVozVmhjbVJmY0dsa0xHOTNiaTV6WlhKMlpYSmZjR2xrTEc5M2JpNWljbTkzYzJWeVgzQnBaQ3dLSUNB"
    "Z0lDQWdJQ0F1TGk0b2IzZHVMbWhsYkhCbGNsOXdhV1E5UFQxdWRXeHNQMXRkT2x0dmQyNHVhR1ZzY0dWeVgzQnBaRjBwWFN3"
    "S0lDQWdJQ0FnYkdGaU9tOTNiaTVzWVdJc2IzVjBaWEpmYjNWME9tOTNiaTV2ZFhSbGNsOXZkWFFzYUdWc2NHVnlYM0JwWkRw"
    "dmQyNHVhR1ZzY0dWeVgzQnBaQ3h6WlhKMlpYSmZjR2xrT205M2JpNXpaWEoyWlhKZmNHbGtDaUFnSUNCOUtUc0tJQ0FnSUdG"
    "amRHbHZiaTVsYm1SZlkyeHZZMnM5Y2k1amJHOWphenRoWTNScGIyNHVjbVZ6ZFd4MFgzTjBZWFJsUFNKeVpYUjFjbTVsWkNJ"
    "N0NpQWdJQ0J5WldOdmNtUXViM2R1WldSZlkyaHBiR1JmWlhocGRITTljaTV2ZDI1bFpGOWphR2xzWkY5bGVHbDBjenNLSUNB"
    "Z0lHUnBZV2R1YjNOMGFXTW9jaTUxYm1WNGNHVmpkR1ZrWDJaaGFXeDFjbVZmYjJKelpYSjJZWFJwYjI0c2NpNTFibVY0Y0dW"
    "amRHVmtYMjlpYzJWeWRtRjBhVzl1WDNCeWIycGxZM1JwYjI0cE93b2dJQ0FnYVdZb1RuVnRZbVZ5TG1selUyRm1aVWx1ZEdW"
    "blpYSW9jaTVvWld4d1pYSmZjR2xrS1NZbWNpNW9aV3h3WlhKZmNHbGtQakFwYjNkdUxtaGxiSEJsY2w5d2FXUTljaTVvWld4"
    "d1pYSmZjR2xrT3dvZ0lDQWdjbVZqYjNKa0xtWnBibUZzWDJGaWMyVnVZMlU5ZTI5M2JtVmtYM0JwWkhNNmNpNXZkMjVsWkY5"
    "d2FXUnpMSEJ2Y25Sek9uSXVjRzl5ZEhNc0NpQWdJQ0FnSUd4aFlqcHlMbXhoWWw5aFluTmxiblFzZDJsdVpHOTNYMk52ZFc1"
    "ME9uSmxZMjl5WkM1M2FXNWtiM2RmWTI5MWJuUS9QMjUxYkd3c0NpQWdJQ0FnSUhObGMzTnBiMjVmWlc1a1pXUTZjbVZqYjNK"
    "a0xuTmxjM05wYjI1ZlpXNWtaV1E5UFQxMGNuVmxmVHNLSUNBZ0lISmxZMjl5WkM1bWFYSnpkRjl2WW5ObGNuWmhkR2x2Ymw5"
    "MGIxOW1hVzVoYkY5M1lXeHNYMjF6UFhKbFkyOXlaQzVtYVhKemRGOXZZbk5sY25aaGRHbHZibDkzWVd4c1gyMXpQVDA5Ym5W"
    "c2JEOXVkV3hzT2dvZ0lDQWdJQ0JFWVhSbExtNXZkeWdwTFhKbFkyOXlaQzVtYVhKemRGOXZZbk5sY25aaGRHbHZibDkzWVd4"
    "c1gyMXpPd29nSUNBZ2NtVjBZV2x1S0NrN0lDOHZJRVoxYkd3Z1ptbDRaV1FnY21WaFpHSmhZMnN2WlhocGRITXZZMnh2WTJz"
    "Z1FrVkdUMUpGSUdOdmJYQmhjbWx6YjI1ekxnb2dJQ0FnYVdZb2NtVmhaRk4wWVhKMEtTQjdDaUFnSUNBZ0lHRmpkR2x2Ymk1"
    "bGJHRndjMlZrWDIxelBVNTFiV0psY2lnb2JuTW9jaTVqYkc5amF5a3Ribk1vY21WaFpGTjBZWEowS1Nrdk1UQXdNREF3TUc0"
    "cE93b2dJQ0FnSUNCaFkzUnBiMjR1YjNabGNsOWlkV1JuWlhROVlXTjBhVzl1TG1Gc2JHOTNaV1JmYlhNOVBUMXVkV3hzUDI1"
    "MWJHdzZZV04wYVc5dUxtVnNZWEJ6WldSZmJYTStZV04wYVc5dUxtRnNiRzkzWldSZmJYTTdDaUFnSUNCOUNpQWdJQ0JwWmlo"
    "eVpXTnZjbVF1Wm1seWMzUmZZMnh2WTJzcENpQWdJQ0FnSUhKbFkyOXlaQzVzWVhSamFGOTBiMTloWW5ObGJtTmxYMjF6UFU1"
    "MWJXSmxjaWdvYm5Nb2NpNWpiRzlqYXlrdGJuTW9jbVZqYjNKa0xtWnBjbk4wWDJOc2IyTnJLU2t2TVRBd01EQXdNRzRwT3dv"
    "Z0lDQWdhV1lvWVdOMGFXOXVMbTkyWlhKZlluVmtaMlYwS1dOc1pXRnVkWEJGY25KdmNpZ2lZMnhsWVc1MWNGOWlkV1JuWlhS"
    "ZlpYaGpaV1ZrWldRaUtUc0tJQ0FnSUdOdmJuTjBJR0U5Y21WamIzSmtMbVpwYm1Gc1gyRmljMlZ1WTJVN0NpQWdJQ0JwWmln"
    "aFkyOXVkSEp2Ykd4bGNrcHZhVzVsWkh4OElVRnljbUY1TG1selFYSnlZWGtvY21WamIzSmtMbTkzYm1Wa1gyTm9hV3hrWDJW"
    "NGFYUnpLWHg4Q2lBZ0lDQWdJQ0J5WldOdmNtUXViM2R1WldSZlkyaHBiR1JmWlhocGRITXViR1Z1WjNSb0lUMDlLRzkzYmk1"
    "b1pXeHdaWEpmY0dsa1BUMDliblZzYkQ4Mk9qY3BLUW9nSUNBZ0lDQmpiR1ZoYm5Wd1JYSnliM0lvSW1Ob2FXeGtYM0psWVhC"
    "ZmFXNWpiMjF3YkdWMFpTSXBPd29nSUNBZ2FXWW9JVTlpYW1WamRDNTJZV3gxWlhNb1lTNXZkMjVsWkY5d2FXUnpLUzVsZG1W"
    "eWVTaDJQVDUyUFQwOWRISjFaU2w4ZkFvZ0lDQWdJQ0FnSVU5aWFtVmpkQzUyWVd4MVpYTW9ZUzV3YjNKMGN5a3VaWFpsY25r"
    "b2RqMCtkajA5UFhSeWRXVXBmSHhoTG14aFlpRTlQWFJ5ZFdWOGZBb2dJQ0FnSUNBZ1lTNTNhVzVrYjNkZlkyOTFiblFoUFQw"
    "d2ZIeGhMbk5sYzNOcGIyNWZaVzVrWldRaFBUMTBjblZsS1FvZ0lDQWdJQ0JqYkdWaGJuVndSWEp5YjNJb0ltOTNibVZrWDNK"
    "bGMyOTFjbU5sWDJGaWMyVnVZMlZmZFc1d2NtOTJaVzRpS1RzS0lDQjlJR05oZEdOb0lIdGhZM1JwYjI0dWNtVnpkV3gwWDNO"
    "MFlYUmxQU0psZUdObGNIUnBiMjRpTzJOc1pXRnVkWEJGY25KdmNpZ2liM2R1WldSZmNtVmhaR0poWTJ0ZmRXNWhkbUZwYkdG"
    "aWJHVWlLVHQ5Q2lBZ1kyeGxZVzUxY0VWeWNtOXlLQ0ptYVhKemRGOWxkbVZ1ZEY5MWJuQnliM1psYmlJcE93b2dJSEpsWTI5"
    "eVpDNTNhRzlzWlY5amJHVmhiblZ3WDNkcGRHaHBiall3WDNCeWIzWmxiajFtWVd4elpUdHlaWFJoYVc0b0tUc0tJQ0F2THlC"
    "RmVHTnNkWE5wZG1VZ2JtVjNJR1pwYkdVZ2IyNXNlVHNnWlhocGMzUnBibWNnYjNWMFpYSXZhR1ZzY0dWeUwzQnliM1pwWkdW"
    "eUlHVjJhV1JsYm1ObElIVnVkRzkxWTJobFpDNEtJQ0IwY25rZ2V3b2dJQ0FnWTI5dWMzUWdZMjFrUFNKd2VYUm9iMjR6SUMx"
    "aklDSXJjM0VvVUVWU1UwbFRWQ2tySWlBaUszTnhLRzkzYmk1amJHVmhiblZ3WDI5MWRDa3JJaUFpSzNOeEtFcFRUMDR1YzNS"
    "eWFXNW5hV1o1S0hKbFkyOXlaQ2twT3dvZ0lDQWdZMjl1YzNRZ2NqMWhkMkZwZENCMGIyOXNjeTVsZUdWalgyTnZiVzFoYm1R"
    "b2UyTnRaQ3gzYjNKclpHbHlPbTkzYmk1M2IzSnJjM0JoWTJVc0NpQWdJQ0FnSUhscFpXeGtYM1JwYldWZmJYTTZNVEF3TURB"
    "c2JXRjRYMjkxZEhCMWRGOTBiMnRsYm5NNk5UQXdmU2s3Q2lBZ0lDQnlaV052Y21RdWJHOWpZV3hmZEc5dmJGOXlaV05sYVhC"
    "MGN5NXdkWE5vS0h0c1lXSmxiRG9pY0dWeWMybHpkQ0lzQ2lBZ0lDQWdJR1Y0YVhRNmRIbHdaVzltSUhJdVpYaHBkRjlqYjJS"
    "bFBUMDlJbTUxYldKbGNpSS9jaTVsZUdsMFgyTnZaR1U2Ym5Wc2JDd0tJQ0FnSUNBZ2MyVnpjMmx2Ymw5cFpEcDBlWEJsYjJZ"
    "Z2NpNXpaWE56YVc5dVgybGtQVDA5SW01MWJXSmxjaUkvY2k1elpYTnphVzl1WDJsa09tNTFiR3g5S1R0eVpYUmhhVzRvS1Rz"
    "S0lDQWdJR2xtS0hJdWMyVnpjMmx2Ymw5cFpDRTlQWFZ1WkdWbWFXNWxaQ2xqYkdWaGJuVndSWEp5YjNJb0ltOTNibVZrWDJ4"
    "dlkyRnNYMk52YlcxaGJtUmZkVzVxYjJsdVpXUWlLVHNLSUNBZ0lHbG1LSEl1YzJWemMybHZibDlwWkNFOVBYVnVaR1ZtYVc1"
    "bFpIeDhjaTVsZUdsMFgyTnZaR1VoUFQwd0tRb2dJQ0FnSUNCamJHVmhiblZ3UlhKeWIzSW9JbU5zWldGdWRYQmZiV1YwWVdS"
    "aGRHRmZkM0pwZEdWZmRXNWpiMjVtYVhKdFpXUWlLVHNLSUNCOUlHTmhkR05vSUh0amJHVmhiblZ3UlhKeWIzSW9JbU5zWldG"
    "dWRYQmZiV1YwWVdSaGRHRmZkM0pwZEdWZmRXNWpiMjVtYVhKdFpXUWlLVHQ5Q2lBZ1kyOXVkSEp2Ykd4bGNrSjFabVpsY2ow"
    "aUlqdHZkMjR1WTJ4dmMyVmtQWFJ5ZFdVN2MzUnZjbVVvSW1Rd01WOXBiVzFsWkdsaGRHVmZiM2R1WldSZmFHRnVaR3hsY3lJ"
    "c2IzZHVLVHNLZlFwaGMzbHVZeUJtZFc1amRHbHZiaUJqYUdWamEyVmtLR3hoWW1Wc0xHOXdaWEpoZEdsdmJpeHdjbVZrYVdO"
    "aGRHVXBJSHNLSUNCcFppaHlaV052Y21RdVptbHljM1JmWm1GcGJIVnlaU0U5UFc1MWJHd3BjbVYwZFhKdUlHNTFiR3c3Q2lB"
    "Z2FXWW9SR0YwWlM1dWIzY29LVDQ5YjNkdUxuTjBZWEowWDJWd2IyTm9YMjF6S3pnME1EQXdNQ2tnZXdvZ0lDQWdiR0YwWTJn"
    "b0ltSnliM2R6WlhKZllXTjBhWFpsWDJSbFlXUnNhVzVsSWl4RVlYUmxMbTV2ZHlncEtUdGhkMkZwZENCamJHVmhiblZ3S0Nr"
    "N2NtVjBkWEp1SUc1MWJHdzdDaUFnZlFvZ0lHeGxkQ0J5WlhOMWJIUXNjbVZqWldsMlpXUlhZV3hzT3dvZ0lIUnllU0I3Q2lB"
    "Z0lDQnlaWE4xYkhROVlYZGhhWFFnYjNCbGNtRjBhVzl1S0NrN2NtVmpaV2wyWldSWFlXeHNQVVJoZEdVdWJtOTNLQ2s3Q2lB"
    "Z0lDQnlaV052Y21RdWIySnpaWEoyWVhScGIyNWZjbVZqWldsd2RITXVjSFZ6YUNoN2EybHVaRHBzWVdKbGJDeHlaV05sYVha"
    "bFpGOTNZV3hzWDIxek9uSmxZMlZwZG1Wa1YyRnNiSDBwTzNKbGRHRnBiaWdwT3dvZ0lDQWdhV1lvY21WemRXeDBQeTVwYzBW"
    "eWNtOXlQVDA5ZEhKMVpTbHNZWFJqYUNnaVluSnZkM05sY2w5MGIyOXNYM0psWm5WelpXUWlMSEpsWTJWcGRtVmtWMkZzYkNr"
    "N0NpQWdJQ0JsYkhObElHbG1LQ0Z3Y21Wa2FXTmhkR1VvY21WemRXeDBLU2xzWVhSamFDZ2lZbkp2ZDNObGNsOWtaV05wYzJs"
    "dmJsOTFibU52Ym1acGNtMWxaQ0lzY21WalpXbDJaV1JYWVd4c0tUc0tJQ0I5SUdOaGRHTm9JSHRzWVhSamFDZ2lZbkp2ZDNO"
    "bGNsOTBiMjlzWDI5eVgyUmxZMmx6YVc5dVgyVjRZMlZ3ZEdsdmJpSXNSR0YwWlM1dWIzY29LU2s3ZlFvZ0lHbG1LSEpsWTI5"
    "eVpDNW1hWEp6ZEY5bVlXbHNkWEpsSVQwOWJuVnNiQ2xoZDJGcGRDQmpiR1ZoYm5Wd0tDazdDaUFnY21WMGRYSnVJSEpsWTI5"
    "eVpDNW1hWEp6ZEY5bVlXbHNkWEpsUFQwOWJuVnNiRDl5WlhOMWJIUTZiblZzYkRzS2ZRcGpiMjV6ZENCemJtRndjMmh2ZEVG"
    "eVozTTllM05sYzNOcGIyNDZiM2R1TG5ObGMzTnBiMjRzZEdGeVoyVjBYMmxrT205M2JpNTBZWEpuWlhSZmFXUXNkR0ZpWDJs"
    "a09tOTNiaTUwWVdKZmFXUXNDaUFnYzI1aGNITm9iM1JmWm05eWJXRjBPaUp6WlcxaGJuUnBZMTkyTWlJc2FXNWpiSFZrWlY5"
    "elkzSmxaVzV6YUc5ME9tWmhiSE5sZlRzS1puVnVZM1JwYjI0Z2NHRm5aU2h5WlhOMWJIUXNhMmx1WkNrZ2V3b2dJR052Ym5O"
    "MElITTljbVZ6ZFd4MFB5NXpkSEoxWTNSMWNtVmtRMjl1ZEdWdWRDeHViMlJsY3oxQmNuSmhlUzVwYzBGeWNtRjVLSE0vTG1O"
    "dmJuUmxiblJmY21WbWN5ay9jeTVqYjI1MFpXNTBYM0psWm5NNlcxMDdDaUFnWTI5dWMzUWdabXhoWjNNOWV3b2dJQ0FnYzNS"
    "aGRIVnpYMjlyT25KbGMzVnNkRDh1YVhORmNuSnZjaUU5UFhSeWRXVW1Kbk0vTG5OMFlYUjFjejA5UFNKdmF5SXNDaUFnSUNC"
    "amIyMXdiR1YwWlRwelB5NXpibUZ3YzJodmREOHVZMjl0Y0d4bGRHVTlQVDEwY25WbExBb2dJQ0FnYUdWaFpHbHVaMTl0WVhS"
    "amFEcHViMlJsY3k1emIyMWxLRzQ5UG00dWNtOXNaVDA5UFNKb1pXRmthVzVuSWlZbWJpNXVZVzFsUFQwOUNpQWdJQ0FnSUNo"
    "cmFXNWtQVDA5SW5CeWIzUmxZM1JsWkY5aFpuUmxjaUkvSWxCeWIzUmxZM1JsWkNCaGNIQnNhV05oZEdsdmJpQmhZMk5sYzNN"
    "aU9pSk1iMk5oYkNCa1pXMXZJaWtwTEFvZ0lDQWdaWEp5YjNKZmJXRjBZMmc2Ym05a1pYTXVjMjl0WlNodVBUNXVMbTVoYldV"
    "OVBUMGlURzlqWVd3Z1pHVnRieUJqYjNWc1pDQnViM1FnWTI5dGNHeGxkR1VnZEdocGN5QnlaWEYxWlhOMExpSXBMQW9nSUNB"
    "Z2NtVnhkV2x5WldSZmRHVjRkRHB1YjJSbGN5NXpiMjFsS0c0OVBtNHVibUZ0WlQwOVBTaHJhVzVrUFQwOUluQnliM1JsWTNS"
    "bFpGOWlaV1p2Y21VaVB5SlRhV2R1SUdsdUlISmxjWFZwY21Wa0xpSTZDaUFnSUNBZ0lHdHBibVE5UFQwaWNISnZkR1ZqZEdW"
    "a1gyRm1kR1Z5SWo4aVUybG5ibVZrSUdsdUxpQlFjbTkwWldOMFpXUWdZWEJ3YkdsallYUnBiMjRnWVdOalpYTnpJR2x6SUdG"
    "MllXbHNZV0pzWlM0aU9nb2dJQ0FnSUNBaVZYTmxJRk5wWjI0Z2FXNGdkRzhnYjNCbGJpQjBhR2x6SUd4dlkyRnNJR0Z3Y0d4"
    "cFkyRjBhVzl1TGlJcEtTd0tJQ0FnSUdsdWRHVnlZV04wYVhabFgzSmxabDl3Y21WelpXNTBPa0Z5Y21GNUxtbHpRWEp5WVhr"
    "b2N6OHVjbVZtY3lrbUpnb2dJQ0FnSUNCekxuSmxabk11YzI5dFpTaHVQVDVCY25KaGVTNXBjMEZ5Y21GNUtHNHVZV04wYVc5"
    "dWN5a21KbTR1WVdOMGFXOXVjeTVwYm1Oc2RXUmxjeWdpWTJ4cFkyc2lLU2tLSUNCOU93b2dJSEpsWTI5eVpDNXNZWE4wWDNC"
    "aFoyVmZabXhoWjNNOWUydHBibVFzTGk0dVpteGhaM045TzNKbGRHRnBiaWdwT3dvZ0lDOHZJRTl1YkhrZ2NISnZkbWxrWlhJ"
    "Z2NtVm1jeUJoYm1RZ1ptbDRaV1FnY0hWaWJHbGpMV3hoWW1Wc0lHMWhkR05vWlhNZ2MzVnlkbWwyWlNCMGFHbHpJSEpoZHlC"
    "emJtRndjMmh2ZEM0S0lDQnZkMjR1Wm5KbGMyaGZjbVZtY3oxQmNuSmhlUzVwYzBGeWNtRjVLSE0vTG5KbFpuTXBQM011Y21W"
    "bWN5NW1hV3gwWlhJb2JqMCtkSGx3Wlc5bUlHNHVjbVZtUFQwOUluTjBjbWx1WnlJbUpnb2dJQ0FnTDE1d1d6QXRPVjByT2xz"
    "d0xUbGRLeVF2TG5SbGMzUW9iaTV5WldZcEtTNXRZWEFvYmowK2JpNXlaV1lwT2x0ZE93b2dJSE4wYjNKbEtDSmtNREZmYVcx"
    "dFpXUnBZWFJsWDI5M2JtVmtYMmhoYm1Sc1pYTWlMRzkzYmlrN0NpQWdhV1lvWm14aFozTXVaWEp5YjNKZmJXRjBZMmdwY21W"
    "MGRYSnVJR1poYkhObE93b2dJR2xtS0NGbWJHRm5jeTV6ZEdGMGRYTmZiMnQ4ZkNGbWJHRm5jeTVqYjIxd2JHVjBaU2x5WlhS"
    "MWNtNGdabUZzYzJVN0NpQWdhV1lvYTJsdVpEMDlQU0puWlc1bGNtbGpYMk5vWldOclpXUWlLU0I3Q2lBZ0lDQnBaaWh1YjJS"
    "bGN5NXpiMjFsS0c0OVBtNHVjbTlzWlQwOVBTSm9aV0ZrYVc1bklpWW1iaTV1WVcxbFBUMDlJbEJ5YjNSbFkzUmxaQ0JoY0hC"
    "c2FXTmhkR2x2YmlCaFkyTmxjM01pS1NZbUNpQWdJQ0FnSUNCdWIyUmxjeTV6YjIxbEtHNDlQbTR1Ym1GdFpUMDlQU0pUYVdk"
    "dVpXUWdhVzR1SUZCeWIzUmxZM1JsWkNCaGNIQnNhV05oZEdsdmJpQmhZMk5sYzNNZ2FYTWdZWFpoYVd4aFlteGxMaUlwS1Fv"
    "Z0lDQWdJQ0J5WldOdmNtUXVjSEp2ZEdWamRHVmtYMkZtZEdWeVgzQmhaMlU5ZEhKMVpUc0tJQ0FnSUhKbGRIVnliaUIwY25W"
    "bE93b2dJSDBLSUNCamIyNXpkQ0J2YXoxbWJHRm5jeTVvWldGa2FXNW5YMjFoZEdOb0ppWm1iR0ZuY3k1eVpYRjFhWEpsWkY5"
    "MFpYaDBKaVlLSUNBZ0lDaHJhVzVrSVQwOUltRndjR3hwWTJGMGFXOXVJbng4Wm14aFozTXVhVzUwWlhKaFkzUnBkbVZmY21W"
    "bVgzQnlaWE5sYm5RcE93b2dJR2xtS0c5ckppWnJhVzVrUFQwOUluQnliM1JsWTNSbFpGOWhablJsY2lJcGNtVmpiM0prTG5C"
    "eWIzUmxZM1JsWkY5aFpuUmxjbDl3WVdkbFBYUnlkV1U3Q2lBZ2NtVjBkWEp1SUc5ck93cDlDbUZ6ZVc1aklHWjFibU4wYVc5"
    "dUlHNWhkbWxuWVhSbFFXNWtVMjVoY0hOb2IzUW9kWEpzTEd0cGJtUXBJSHNLSUNCcFppaGhkMkZwZENCamFHVmphMlZrS0NK"
    "aWNtOTNjMlZ5WDI1aGRtbG5ZWFJwYjI0aUxBb2dJQ0FnSUNBb0tUMCtkRzl2YkhNdWJXTndYMTlqZFdGZlpISnBkbVZ5WDE5"
    "aWNtOTNjMlZ5WDI1aGRtbG5ZWFJsS0h0elpYTnphVzl1T205M2JpNXpaWE56YVc5dUxBb2dJQ0FnSUNBZ0lIUmhjbWRsZEY5"
    "cFpEcHZkMjR1ZEdGeVoyVjBYMmxrTEhSaFlsOXBaRHB2ZDI0dWRHRmlYMmxrTEhWeWJIMHBMQW9nSUNBZ0lDQnlQVDV5UHk1"
    "cGMwVnljbTl5SVQwOWRISjFaU2twSUhzS0lDQWdJR0YzWVdsMElHTm9aV05yWldRb0ltSnliM2R6WlhKZmMyNWhjSE5vYjNR"
    "aUxBb2dJQ0FnSUNBb0tUMCtkRzl2YkhNdWJXTndYMTlqZFdGZlpISnBkbVZ5WDE5blpYUmZZbkp2ZDNObGNsOXpkR0YwWlNo"
    "emJtRndjMmh2ZEVGeVozTXBMSEk5UG5CaFoyVW9jaXhyYVc1a0tTazdDaUFnZlFwOUNtRnplVzVqSUdaMWJtTjBhVzl1SUdW"
    "dWRISjVLQ2tnZXdvZ0lDOHZJRlJvWlNCbGVHRmpkQ0JFY21sMlpYSXRiM2R1WldRZ1lteGhibXNnYUdGdVpHeGxjeUJoYkhK"
    "bFlXUjVJR1Y0YVhOMElIZG9hV3hsSUhSb1pURTRNSE1nWjJGMFpTQjNZV2wwY3k0S0lDQXZMeUJVYUdWeVpTQnBjeUJ1YnlC"
    "dGIyUmxiQ0I1YVdWc1pDd2dkRzl2YkMxa1pYTmpjbWx3ZEdsdmJpQnNiMkZrSUc5eUlISmxZbWx1WkNCaFpuUmxjaUIwYUds"
    "eklHMWhjbXRsY2k0S0lDQmpiMjV6ZENCdFlYSnJaWEpEYjIxdFlXNWtQU0p3ZVhSb2IyNHpJQzFqSUNJcmMzRW9UVUZTUzBW"
    "U0tTc2lJQ0lyYzNFb2IzZHVMbXhoWWlrcklpQWlLd29nSUNBZ2MzRW9iM2R1TG1kMVlYSmtYM0JwWkNrcklpQWlLM054S0c5"
    "M2JpNXpaWEoyWlhKZmNHbGtLVHNLSUNCaGQyRnBkQ0JqYUdWamEyVmtLQ0p3Y21Wd1lYSmxaRjl0WVhKclpYSWlMQW9nSUNB"
    "Z0tDazlQblJ2YjJ4ekxtVjRaV05mWTI5dGJXRnVaQ2g3WTIxa09tMWhjbXRsY2tOdmJXMWhibVFzZDI5eWEyUnBjanB2ZDI0"
    "dWQyOXlhM053WVdObExBb2dJQ0FnSUNCNWFXVnNaRjkwYVcxbFgyMXpPakV3TURBd0xHMWhlRjl2ZFhSd2RYUmZkRzlyWlc1"
    "ek9qVXdNSDBwTEhJOVBuc0tJQ0FnSUNBZ2NtVmpiM0prTG14dlkyRnNYM1J2YjJ4ZmNtVmpaV2x3ZEhNdWNIVnphQ2g3YkdG"
    "aVpXdzZJbTFoY210bGNpSXNDaUFnSUNBZ0lDQWdaWGhwZERwMGVYQmxiMllnY2k1bGVHbDBYMk52WkdVOVBUMGliblZ0WW1W"
    "eUlqOXlMbVY0YVhSZlkyOWtaVHB1ZFd4c0xBb2dJQ0FnSUNBZ0lITmxjM05wYjI1ZmFXUTZkSGx3Wlc5bUlISXVjMlZ6YzJs"
    "dmJsOXBaRDA5UFNKdWRXMWlaWElpUDNJdWMyVnpjMmx2Ymw5cFpEcHVkV3hzZlNrN2NtVjBZV2x1S0NrN0NpQWdJQ0FnSUds"
    "bUtISXVjMlZ6YzJsdmJsOXBaQ0U5UFhWdVpHVm1hVzVsWkNsamJHVmhiblZ3UlhKeWIzSW9JbTkzYm1Wa1gyeHZZMkZzWDJO"
    "dmJXMWhibVJmZFc1cWIybHVaV1FpS1RzS0lDQWdJQ0FnY21WMGRYSnVJSEl1WlhocGRGOWpiMlJsUFQwOU1DWW1jaTV6WlhO"
    "emFXOXVYMmxrUFQwOWRXNWtaV1pwYm1Wa0ppWUtJQ0FnSUNBZ0lDQktVMDlPTG5CaGNuTmxLSEl1YjNWMGNIVjBLUzV3Y21W"
    "d1lYSmxaRjl0WVhKclpYSmZkM0pwZEhSbGJqMDlQWFJ5ZFdVN0NpQWdJQ0I5S1RzS0lDQmpiMjV6ZENCeVpXRmtlVVJsWVdS"
    "c2FXNWxQVVJoZEdVdWJtOTNLQ2tyTXpBd01EQTdDaUFnZDJocGJHVW9jbVZqYjNKa0xtWnBjbk4wWDJaaGFXeDFjbVU5UFQx"
    "dWRXeHNKaVp2ZDI0dVptbDRkSFZ5WlY5eVpXRmtlU0U5UFhSeWRXVW1Ka1JoZEdVdWJtOTNLQ2s4Y21WaFpIbEVaV0ZrYkds"
    "dVpTa0tJQ0FnSUdGM1lXbDBJSEJ2Ykd4RGIyNTBjbTlzYkdWeUtDazdDaUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4"
    "MWNtVTlQVDF1ZFd4c0ppWnZkMjR1Wm1sNGRIVnlaVjl5WldGa2VTRTlQWFJ5ZFdVcENpQWdJQ0JzWVhSamFDZ2labWw0ZEhW"
    "eVpWOXlaV0ZrZVY5MWJtTnZibVpwY20xbFpDSXNSR0YwWlM1dWIzY29LU2s3Q2lBZ2FXWW9jbVZqYjNKa0xtWnBjbk4wWDJa"
    "aGFXeDFjbVVoUFQxdWRXeHNLWHRoZDJGcGRDQmpiR1ZoYm5Wd0tDazdjbVYwZFhKdU8zMEtJQ0JoZDJGcGRDQndiMnhzUTI5"
    "dWRISnZiR3hsY2lncE95QXZMeUJQVGtVZ2FXMXRaV1JwWVhSbElIQnlaUzF1WVhacFoyRjBhVzl1SUhCdmJHd3NJRzV2SUZK"
    "UUlFaFVWRkFnY0hKdlltVXVDaUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVWhQVDF1ZFd4c0tYdGhkMkZwZENC"
    "amJHVmhiblZ3S0NrN2NtVjBkWEp1TzMwS0lDQmhkMkZwZENCdVlYWnBaMkYwWlVGdVpGTnVZWEJ6YUc5MEtDSm9kSFJ3T2k4"
    "dmJHOWpZV3hvYjNOME9qTXdNREF2Y0hKdmRHVmpkR1ZrSWl3aWNISnZkR1ZqZEdWa1gySmxabTl5WlNJcE93b2dJR2xtS0hK"
    "bFkyOXlaQzVtYVhKemRGOW1ZV2xzZFhKbFBUMDliblZzYkNsaGQyRnBkQ0J3YjJ4c1EyOXVkSEp2Ykd4bGNpZ3BPd29nSUds"
    "bUtISmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxJVDA5Ym5Wc2JDbDdZWGRoYVhRZ1kyeGxZVzUxY0NncE8zSmxkSFZ5Ymp0"
    "OUNpQWdZWGRoYVhRZ2JtRjJhV2RoZEdWQmJtUlRibUZ3YzJodmRDZ2lhSFIwY0RvdkwyeHZZMkZzYUc5emREb3pNREF3THlJ"
    "c0ltRndjR3hwWTJGMGFXOXVJaWs3Q2lBZ2FXWW9jbVZqYjNKa0xtWnBjbk4wWDJaaGFXeDFjbVU5UFQxdWRXeHNLV0YzWVds"
    "MElIQnZiR3hEYjI1MGNtOXNiR1Z5S0NrN0lDOHZJRTlPUlNCd2IzTjBMV1Z1ZEhKNUlIQnZiR3d1Q2lBZ2FXWW9jbVZqYjNK"
    "a0xtWnBjbk4wWDJaaGFXeDFjbVU5UFQxdWRXeHNLU0I3Q2lBZ0lDQnZkMjR1Y0doaGMyVTlJbUp5YjNkelpYSmZaR1ZqYVhO"
    "cGIyNGlPd29nSUNBZ2MzUnZjbVVvSW1Rd01WOXBiVzFsWkdsaGRHVmZiM2R1WldSZmFHRnVaR3hsY3lJc2IzZHVLVHNLSUNC"
    "OUNpQWdhV1lvY21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21VaFBUMXVkV3hzS1dGM1lXbDBJR05zWldGdWRYQW9LVHNLZlFw"
    "aGMzbHVZeUJtZFc1amRHbHZiaUJqYjI1MGFXNTFZWFJwYjI0b0tTQjdDaUFnWTI5dWMzUWdjM0JsWXoxdmQyNHVibVY0ZEY5"
    "a1pXTnBjMmx2YmpzS0lDQmpiMjV6ZENCaGJHeHZkMlZrUzJsdVpITTlXeUpqYkdsamF5SXNJblZ6WlhKdVlXMWxJaXdpY0dG"
    "emMzZHZjbVFpTENKemJtRndjMmh2ZENJc0luQnliM1JsWTNSbFpGOWhablJsY2lKZE93b2dJR2xtS0NGemNHVmpmSHdoWVd4"
    "c2IzZGxaRXRwYm1SekxtbHVZMngxWkdWektITndaV011YTJsdVpDa3BJSHNLSUNBZ0lHeGhkR05vS0NKaWNtOTNjMlZ5WDJS"
    "bFkybHphVzl1WDNWdVkyOXVabWx5YldWa0lpeEVZWFJsTG01dmR5Z3BLVHRoZDJGcGRDQmpiR1ZoYm5Wd0tDazdjbVYwZFhK"
    "dU93b2dJSDBLSUNCaGQyRnBkQ0J3YjJ4c1EyOXVkSEp2Ykd4bGNpZ3BPd29nSUdsbUtISmxZMjl5WkM1bWFYSnpkRjltWVds"
    "c2RYSmxJVDA5Ym5Wc2JDbDdZWGRoYVhRZ1kyeGxZVzUxY0NncE8zSmxkSFZ5Ymp0OUNpQWdhV1lvYzNCbFl5NXJhVzVrUFQw"
    "OUluQnliM1JsWTNSbFpGOWhablJsY2lJcElIc0tJQ0FnSUM4dklGSmxZV1FnZEdobElHWnlaWE5vSUdOaGJHeGlZV05yTFda"
    "dmJHeHZkMmx1WnlCd2NtOTBaV04wWldRZ2NHRm5aVHNnWkc4Z2JtOTBJSE5sYm1RZ1lXNXZkR2hsY2lCU1VDQnlaWEYxWlhO"
    "MExnb2dJQ0FnWVhkaGFYUWdZMmhsWTJ0bFpDZ2lZbkp2ZDNObGNsOXpibUZ3YzJodmRDSXNDaUFnSUNBZ0lDZ3BQVDUwYjI5"
    "c2N5NXRZM0JmWDJOMVlWOWtjbWwyWlhKZlgyZGxkRjlpY205M2MyVnlYM04wWVhSbEtITnVZWEJ6YUc5MFFYSm5jeWtzY2ow"
    "K2NHRm5aU2h5TENKd2NtOTBaV04wWldSZllXWjBaWElpS1NrN0NpQWdmU0JsYkhObElHbG1LSE53WldNdWEybHVaRDA5UFNK"
    "emJtRndjMmh2ZENJcElIc0tJQ0FnSUdGM1lXbDBJR05vWldOclpXUW9JbUp5YjNkelpYSmZjMjVoY0hOb2IzUWlMQW9nSUNB"
    "Z0lDQW9LVDArZEc5dmJITXViV053WDE5amRXRmZaSEpwZG1WeVgxOW5aWFJmWW5KdmQzTmxjbDl6ZEdGMFpTaHpibUZ3YzJo"
    "dmRFRnlaM01wTEFvZ0lDQWdJQ0J5UFQ1d1lXZGxLSElzSW1kbGJtVnlhV05mWTJobFkydGxaQ0lwSmlad2RXSnNhV05RY21W"
    "a2FXTmhkR1VvY2l4emNHVmpLU2s3Q2lBZ2ZTQmxiSE5sSUhzS0lDQWdJR2xtS0hSNWNHVnZaaUJ6Y0dWakxuSmxaaUU5UFNK"
    "emRISnBibWNpZkh3aFFYSnlZWGt1YVhOQmNuSmhlU2h2ZDI0dVpuSmxjMmhmY21WbWN5bDhmQW9nSUNBZ0lDQWdJVzkzYmk1"
    "bWNtVnphRjl5WldaekxtbHVZMngxWkdWektITndaV011Y21WbUtTa2dld29nSUNBZ0lDQnNZWFJqYUNnaVluSnZkM05sY2w5"
    "a1pXTnBjMmx2Ymw5MWJtTnZibVpwY20xbFpDSXNSR0YwWlM1dWIzY29LU2s3WVhkaGFYUWdZMnhsWVc1MWNDZ3BPM0psZEhW"
    "eWJqc0tJQ0FnSUgwS0lDQWdJRzkzYmk1bWNtVnphRjl5WldaelBWdGRPM04wYjNKbEtDSmtNREZmYVcxdFpXUnBZWFJsWDI5"
    "M2JtVmtYMmhoYm1Sc1pYTWlMRzkzYmlrN0lDOHZJRk5wYm1kc1pTMTFjMlVnYzI1aGNITm9iM1FnY21WbUxnb2dJQ0FnWTI5"
    "dWMzUWdZWEpuY3oxN2MyVnpjMmx2YmpwdmQyNHVjMlZ6YzJsdmJpeDBZWEpuWlhSZmFXUTZiM2R1TG5SaGNtZGxkRjlwWkN4"
    "MFlXSmZhV1E2YjNkdUxuUmhZbDlwWkN4eVpXWTZjM0JsWXk1eVpXWjlPd29nSUNBZ1kyOXVjM1FnYjNCbGNtRjBhVzl1UFhO"
    "d1pXTXVhMmx1WkQwOVBTSmpiR2xqYXlJL0NpQWdJQ0FnSUNncFBUNTBiMjlzY3k1dFkzQmZYMk4xWVY5a2NtbDJaWEpmWDJK"
    "eWIzZHpaWEpmWTJ4cFkyc29leTR1TG1GeVozTXNhVzV3ZFhSZmNtOTFkR1U2SW1SdmJWOWxkbVZ1ZENKOUtUb0tJQ0FnSUNB"
    "Z0tDazlQblJ2YjJ4ekxtMWpjRjlmWTNWaFgyUnlhWFpsY2w5ZlluSnZkM05sY2w5MGVYQmxLSHN1TGk1aGNtZHpMSEpsY0d4"
    "aFkyVTZkSEoxWlN3S0lDQWdJQ0FnSUNCMFpYaDBPbk53WldNdWEybHVaRDA5UFNKMWMyVnlibUZ0WlNJL0ltRmtiV2x1SWpw"
    "c2IyRmtLQ0prTURGZlpuSmxjMmhmY0dGemMzZHZjbVJmYVc1d2RYUWlLWDBwT3dvZ0lDQWdhV1lvYzNCbFl5NXJhVzVrUFQw"
    "OUluQmhjM04zYjNKa0lpWW1LSFI1Y0dWdlppQnNiMkZrS0NKa01ERmZabkpsYzJoZmNHRnpjM2R2Y21SZmFXNXdkWFFpS1NF"
    "OVBTSnpkSEpwYm1jaWZId0tJQ0FnSUNBZ0lDRXZYbHRCTFZwaExYb3dMVGxmTFYxN016QXNNVEk0ZlNRdkxuUmxjM1FvYkc5"
    "aFpDZ2laREF4WDJaeVpYTm9YM0JoYzNOM2IzSmtYMmx1Y0hWMElpa3BLU2tnZXdvZ0lDQWdJQ0JzWVhSamFDZ2lZbkp2ZDNO"
    "bGNsOWtaV05wYzJsdmJsOTFibU52Ym1acGNtMWxaQ0lzUkdGMFpTNXViM2NvS1NrN1lYZGhhWFFnWTJ4bFlXNTFjQ2dwTzNK"
    "bGRIVnlianNLSUNBZ0lIMEtJQ0FnSUdGM1lXbDBJR05vWldOclpXUW9JbUp5YjNkelpYSmZhVzV3ZFhRaUxHOXdaWEpoZEds"
    "dmJpeHlQVDU3Q2lBZ0lDQWdJR052Ym5OMElITTljajh1YzNSeWRXTjBkWEpsWkVOdmJuUmxiblE3Q2lBZ0lDQWdJSEpsZEhW"
    "eWJpQnlQeTVwYzBWeWNtOXlJVDA5ZEhKMVpTWW1XeUpqYjI1bWFYSnRaV1FpTENKMWJuWmxjbWxtYVdGaWJHVWlYUzVwYm1O"
    "c2RXUmxjeWh6UHk1bFptWmxZM1FwT3dvZ0lDQWdmU2s3SUM4dklFUnBjM0JoZEdOb0lHRnNiMjVsSUc1bGRtVnlJR1ZoY201"
    "eklHRnVJR0Z3Y0d4cFkyRjBhVzl1SUc5MWRHTnZiV1VnYjNJZ2FtOTFjbTVsZVNCamNtVmthWFF1Q2lBZ0lDQnBaaWh6Y0dW"
    "akxtdHBibVE5UFQwaWNHRnpjM2R2Y21RaUtYTjBiM0psS0NKa01ERmZabkpsYzJoZmNHRnpjM2R2Y21SZmFXNXdkWFFpTEc1"
    "MWJHd3BPd29nSUNBZ2FXWW9jbVZqYjNKa0xtWnBjbk4wWDJaaGFXeDFjbVU5UFQxdWRXeHNLUW9nSUNBZ0lDQmhkMkZwZENC"
    "amFHVmphMlZrS0NKaWNtOTNjMlZ5WDNOdVlYQnphRzkwSWl3S0lDQWdJQ0FnSUNBb0tUMCtkRzl2YkhNdWJXTndYMTlqZFdG"
    "ZlpISnBkbVZ5WDE5blpYUmZZbkp2ZDNObGNsOXpkR0YwWlNoemJtRndjMmh2ZEVGeVozTXBMQW9nSUNBZ0lDQWdJSEk5UG5C"
    "aFoyVW9jaXdpWjJWdVpYSnBZMTlqYUdWamEyVmtJaWttSm5CMVlteHBZMUJ5WldScFkyRjBaU2h5TEhOd1pXTXBLVHNLSUNC"
    "OUNpQWdhV1lvY21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21VOVBUMXVkV3hzS1dGM1lXbDBJSEJ2Ykd4RGIyNTBjbTlzYkdW"
    "eUtDazdDaUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVWhQVDF1ZFd4c0tXRjNZV2wwSUdOc1pXRnVkWEFvS1Rz"
    "S0lDQmxiSE5sSUdsbUtITndaV011YTJsdVpEMDlQU0p3Y205MFpXTjBaV1JmWVdaMFpYSWlLU0I3Q2lBZ0lDQnlaV052Y21R"
    "dVkyeGxZVzUxY0Y5eVpYRjFaWE4wWldSZmQyRnNiRjl0Y3oxRVlYUmxMbTV2ZHlncE8zSmxkR0ZwYmlncE93b2dJQ0FnWVhk"
    "aGFYUWdZMnhsWVc1MWNDZ3BPd29nSUgwS2ZRcG1kVzVqZEdsdmJpQndkV0pzYVdOUWNtVmthV05oZEdVb2NtVnpkV3gwTEhO"
    "d1pXTXBJSHNLSUNCamIyNXpkQ0J1YjJSbGN6MXlaWE4xYkhRL0xuTjBjblZqZEhWeVpXUkRiMjUwWlc1MFB5NWpiMjUwWlc1"
    "MFgzSmxabk03Q2lBZ0x5OGdVbTl2ZENCdGRYTjBJSEJwYmlCdmJtVWdjSEpwYm5SbFpDQndkV0pzYVdNZ1pYaHdaV04wWldR"
    "Z2JHRmlaV3d2Y205c1pTQmlaV1p2Y21VZ2RHaGhkQ0JoWTNScGIyNHVDaUFnTHk4Z1RtOGdjbUYzSUdacFpXeGtJSFpoYkhW"
    "bExDQlZVa3dzSUhOMVltcGxZM1FzSUhGMVpYSjVJRzl5SUdGeVltbDBjbUZ5ZVNCeVpXZGxlQ0J3Y21Wa2FXTmhkR1VnYVhN"
    "Z1lXeHNiM2RsWkM0S0lDQmpiMjV6ZENCeWIyeGxjejFiSW1obFlXUnBibWNpTENKaWRYUjBiMjRpTENKemRHRjBhV04wWlho"
    "MElpd2lkR1Y0ZEdKdmVDSmRPd29nSUdOdmJuTjBJR3hoWW1Wc2N6MWJJbFZ6WlhKdVlXMWxJaXdpVUdGemMzZHZjbVFpTENK"
    "QmRYUm9aVzUwYVdOaGRHOXlJRzl5SUhKbFkyOTJaWEo1SUdOdlpHVWlMQ0pUYVdkdUlHbHVJaXdLSUNBZ0lDSk1iMk5oYkNC"
    "a1pXMXZJaXdpVUhKdmRHVmpkR1ZrSUdGd2NHeHBZMkYwYVc5dUlHRmpZMlZ6Y3lJc0NpQWdJQ0FpVTJsbmJtVmtJR2x1TGlC"
    "UWNtOTBaV04wWldRZ1lYQndiR2xqWVhScGIyNGdZV05qWlhOeklHbHpJR0YyWVdsc1lXSnNaUzRpWFRzS0lDQnlaWFIxY200"
    "Z1FYSnlZWGt1YVhOQmNuSmhlU2h1YjJSbGN5a21Kbkp2YkdWekxtbHVZMngxWkdWektITndaV011Wlhod1pXTjBaV1JmY205"
    "c1pTa21KZ29nSUNBZ2JHRmlaV3h6TG1sdVkyeDFaR1Z6S0hOd1pXTXVaWGh3WldOMFpXUmZiR0ZpWld3cEppWUtJQ0FnSUc1"
    "dlpHVnpMbk52YldVb2JqMCtiaTV5YjJ4bFBUMDljM0JsWXk1bGVIQmxZM1JsWkY5eWIyeGxKaVp1TG01aGJXVTlQVDF6Y0dW"
    "akxtVjRjR1ZqZEdWa1gyeGhZbVZzS1RzS2ZRcDBjbmtnZXdvZ0lHbG1LRzkzYmk1d2FHRnpaVDA5UFNKd2NtVndZWEpsWkY5"
    "bGJuUnllU0lwWVhkaGFYUWdaVzUwY25rb0tUdGxiSE5sSUdGM1lXbDBJR052Ym5ScGJuVmhkR2x2YmlncE93b2dJQzh2SUVS"
    "dklHNXZkQ0JqWVhKeWVTQjFiblpoYkdsa1lYUmxaQ0J3WVhKMGFXRnNJR052Ym5SeWIyeHNaWElnZEdWNGRDQmhZM0p2YzNN"
    "Z2RHaHBjeUJqWld4c0lHSnZkVzVrWVhKNUxnb2dJSGRvYVd4bEtHTnZiblJ5YjJ4c1pYSkNkV1ptWlhJdWJHVnVaM1JvUGpB"
    "bUpuSmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxQVDA5Ym5Wc2JDa2dld29nSUNBZ1kyOXVjM1FnWW1WbWIzSmxVRzlzYkZk"
    "aGJHdzlSR0YwWlM1dWIzY29LVHNLSUNBZ0lHbG1LR0psWm05eVpWQnZiR3hYWVd4c1BqMXZkMjR1YzNSaGNuUmZaWEJ2WTJo"
    "ZmJYTXJPRFF3TURBd0tTQjdDaUFnSUNBZ0lHeGhkR05vS0NKaWNtOTNjMlZ5WDJGamRHbDJaVjlrWldGa2JHbHVaU0lzWW1W"
    "bWIzSmxVRzlzYkZkaGJHd3BPMkp5WldGck93b2dJQ0FnZlFvZ0lDQWdhV1lvWTI5dWRISnZiR3hsY2twdmFXNWxaSHg4SVdO"
    "dmJuUnliMnhzWlhKUWIyeHNRWFpoYVd4aFlteGxLU0I3Q2lBZ0lDQWdJR3hoZEdOb0tDSmpiMjUwY205c2JHVnlYMjlpYzJW"
    "eWRtRjBhVzl1WDJsdWRtRnNhV1FpTEdKbFptOXlaVkJ2Ykd4WFlXeHNLVHRpY21WaGF6c0tJQ0FnSUgwS0lDQWdJR0YzWVds"
    "MElIQnZiR3hEYjI1MGNtOXNiR1Z5S0NrN0lDOHZJRk5oYldVZ2IzZHVaV1FnYzJWemMybHZianNnYjJKelpYSjJaWElnY21W"
    "MFlXbHVjeUJ5WldObGFYQjBJR1pwY25OMExnb2dJQ0FnWTI5dWMzUWdZV1owWlhKUWIyeHNWMkZzYkQxRVlYUmxMbTV2ZHln"
    "cE93b2dJQ0FnYVdZb1lXWjBaWEpRYjJ4c1YyRnNiRDQ5YjNkdUxuTjBZWEowWDJWd2IyTm9YMjF6S3pnME1EQXdNQ2tLSUNB"
    "Z0lDQWdiR0YwWTJnb0ltSnliM2R6WlhKZllXTjBhWFpsWDJSbFlXUnNhVzVsSWl4aFpuUmxjbEJ2Ykd4WFlXeHNLVHNLSUNC"
    "OUNpQWdhV1lvY21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21VaFBUMXVkV3hzS1dGM1lXbDBJR05zWldGdWRYQW9LVHNLZlNC"
    "allYUmphQ0I3Q2lBZ2JHRjBZMmdvSW1OdmJYQnZjMlZrWDJSbFkybHphVzl1WDJWNFkyVndkR2x2YmlJc1JHRjBaUzV1YjNj"
    "b0tTazdDaUFnWVhkaGFYUWdZMnhsWVc1MWNDZ3BPd3A5Q21sbUtISmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxJVDA5Ym5W"
    "c2JDa2dld29nSUhSbGVIUW9lM0psYzNWc2REb2labUZwYkdWa0lpeG1hWEp6ZEY5bVlXbHNkWEpsT25KbFkyOXlaQzVtYVhK"
    "emRGOW1ZV2xzZFhKbExBb2dJQ0FnZFc1bGVIQmxZM1JsWkY5bVlXbHNkWEpsWDI5aWMyVnlkbUYwYVc5dU9uSmxZMjl5WkM1"
    "MWJtVjRjR1ZqZEdWa1gyWmhhV3gxY21WZmIySnpaWEoyWVhScGIyNHNDaUFnSUNCa2FXRm5ibTl6ZEdsalgyVnljbTl5Y3pw"
    "eVpXTnZjbVF1WkdsaFoyNXZjM1JwWTE5bGNuSnZjbk1zWTJ4bFlXNTFjRjlsY25KdmNuTTZjbVZqYjNKa0xtTnNaV0Z1ZFhC"
    "ZlpYSnliM0p6TEFvZ0lDQWdabWx1WVd4ZllXSnpaVzVqWlRweVpXTnZjbVF1Wm1sdVlXeGZZV0p6Wlc1alpTeGpiMjUwY205"
    "c2JHVnlYMlY0YVhRNmNtVmpiM0prTG1OdmJuUnliMnhzWlhKZlpYaHBkQ3dLSUNBZ0lIZG9iMnhsWDJOc1pXRnVkWEJmZDJs"
    "MGFHbHVOakJmY0hKdmRtVnVPbVpoYkhObExBb2dJQ0FnY21WemIzVnlZMlZmY21Wc1pXRnpaVjl3Y205MlpXNDZjbVZqYjNK"
    "a0xtWnBibUZzWDJGaWMyVnVZMlVoUFQxdWRXeHNKaVlLSUNBZ0lDQWdJWEpsWTI5eVpDNWpiR1ZoYm5Wd1gyVnljbTl5Y3k1"
    "emIyMWxLSFk5UGxzS0lDQWdJQ0FnSUNBaVkyaHBiR1JmY21WaGNGOXBibU52YlhCc1pYUmxJaXdpYjNkdVpXUmZjbVZ6YjNW"
    "eVkyVmZZV0p6Wlc1alpWOTFibkJ5YjNabGJpSXNDaUFnSUNBZ0lDQWdJbTkzYm1Wa1gzSmxZV1JpWVdOclgzVnVZWFpoYVd4"
    "aFlteGxJaXdpYjNkdVpXUmZiRzlqWVd4ZlkyOXRiV0Z1WkY5MWJtcHZhVzVsWkNKZExtbHVZMngxWkdWektIWXBLWDBwT3dw"
    "OUlHVnNjMlVnZXdvZ0lIUmxlSFFvZTNKbGMzVnNkRG9pWW5KdmQzTmxjbDlrWldOcGMybHZibDl2WW5ObGNuWmxaRjl2Ym14"
    "NUlpeHdZV2RsWDJac1lXZHpPbkpsWTI5eVpDNXNZWE4wWDNCaFoyVmZabXhoWjNNL1AyNTFiR3dzQ2lBZ0lDQnFiM1Z5Ym1W"
    "NVgyTnlaV1JwZERwbVlXeHpaU3hqYkdWaGJuVndYMlZ5Y205eWN6cHlaV052Y21RdVkyeGxZVzUxY0Y5bGNuSnZjbk1zQ2lB"
    "Z0lDQnlaWE52ZFhKalpWOXlaV3hsWVhObFgzQnliM1psYmpweVpXTnZjbVF1WTJ4bFlXNTFjRjl6ZEdGeWRHVmtQVDA5ZEhK"
    "MVpTWW1jbVZqYjNKa0xtWnBibUZzWDJGaWMyVnVZMlVoUFQxdWRXeHNKaVlLSUNBZ0lDQWdJWEpsWTI5eVpDNWpiR1ZoYm5W"
    "d1gyVnljbTl5Y3k1emIyMWxLSFk5UGxzS0lDQWdJQ0FnSUNBaVkyaHBiR1JmY21WaGNGOXBibU52YlhCc1pYUmxJaXdpYjNk"
    "dVpXUmZjbVZ6YjNWeVkyVmZZV0p6Wlc1alpWOTFibkJ5YjNabGJpSXNDaUFnSUNBZ0lDQWdJbTkzYm1Wa1gzSmxZV1JpWVdO"
    "clgzVnVZWFpoYVd4aFlteGxJaXdpYjNkdVpXUmZiRzlqWVd4ZlkyOXRiV0Z1WkY5MWJtcHZhVzVsWkNKZExtbHVZMngxWkdW"
    "ektIWXBLU3dLSUNBZ0lIZG9iMnhsWDJOc1pXRnVkWEJmZDJsMGFHbHVOakJmY0hKdmRtVnVPbVpoYkhObGZTazdDbjBLIiwK"
    "ICAiYmFzZWxpbmVfYjY0IjogIkx5OGdVSEp2YzNCbFkzUnBkbVVnVDA1RklHWjFibU4wYVc5dWN5NWxlR1ZqSUdObGJHdzdJ"
    "RTVQVkNCbGVHVmpkWFJsWkNCcGJpQjBhR2x6SUdSbGMybG5iaUJ3YUdGelpTNEtZMjl1YzNRZ1EweFBRMHM5SW1sdGNHOXlk"
    "Q0JxYzI5dUxIUnBiV1ZjYm5CeWFXNTBLR3B6YjI0dVpIVnRjSE1vZXlkdGIyNXZkRzl1YVdOZmJuTW5Pbk4wY2loMGFXMWxM"
    "bTF2Ym05MGIyNXBZMTl1Y3lncEtTd25kMkZzYkY5bGNHOWphRjl1Y3ljNmMzUnlLSFJwYldVdWRHbHRaVjl1Y3lncEtYMHBL"
    "Vnh1SWpzS1kyOXVjM1FnVTFSUFVEMGlhVzF3YjNKMElHcHpiMjRzYjNNc2NHRjBhR3hwWWl4emRHRjBMSE41Y3l4MGFXMWxY"
    "RzVqUFdwemIyNHViRzloWkhNb2MzbHpMbUZ5WjNaYk1WMHBYRzVqYkc5amF6MTdKMjF2Ym05MGIyNXBZMTl1Y3ljNmMzUnlL"
    "SFJwYldVdWJXOXViM1J2Ym1salgyNXpLQ2twTENkM1lXeHNYMlZ3YjJOb1gyNXpKenB6ZEhJb2RHbHRaUzUwYVcxbFgyNXpL"
    "Q2twZlZ4dWNtVmpiM0prUFhzbmMyTm9aVzFoSnpvbmNtbGhkWFJvTG1Rd01TMW1hWEp6ZEMxdlluTmxjblpoZEdsdmJpOTJN"
    "U2NzSjJacGNuTjBYMlpoYVd4MWNtVW5PbU5iSjJacGNuTjBYMlpoYVd4MWNtVW5YU3duWm1seWMzUmZiMkp6WlhKMllYUnBi"
    "MjVmZDJGc2JGOXRjeWM2WTFzblptbHljM1JmYjJKelpYSjJZWFJwYjI1ZmQyRnNiRjl0Y3lkZExDZG1hWEp6ZEY5bGRtVnVk"
    "Rjl3Y205MlpXNG5Pa1poYkhObExDZGpiRzlqYXljNlkyeHZZMnQ5WEc1bGRtVnVkRjl6ZEdGMFpUMG5kM0pwZEdWZmRXNWpi"
    "MjVtYVhKdFpXUW5YRzUwY25rNlhHNGdJQ0FnWm1ROWIzTXViM0JsYmlod1lYUm9iR2xpTGxCaGRHZ29ZMXNuWlhabGJuUmZi"
    "M1YwSjEwcExHOXpMazlmVjFKUFRreFpmRzl6TGs5ZlExSkZRVlI4YjNNdVQxOUZXRU5NZkc5ekxrOWZUazlHVDB4TVQxY3NN"
    "RzgyTURBcFhHNGdJQ0FnZDJsMGFDQnZjeTVtWkc5d1pXNG9abVFzSjNjbkxHVnVZMjlrYVc1blBTZGhjMk5wYVNjcElHRnpJ"
    "R1k2WEc0Z0lDQWdJQ0FnSUdZdWQzSnBkR1VvYW5OdmJpNWtkVzF3Y3loeVpXTnZjbVFzYzI5eWRGOXJaWGx6UFZSeWRXVXBL"
    "eWRjWEc0bktUdG1MbVpzZFhOb0tDazdiM011Wm5ONWJtTW9aaTVtYVd4bGJtOG9LU2xjYmlBZ0lDQmxkbVZ1ZEY5emRHRjBa"
    "VDBuZDNKcGRIUmxiaWRjYm1WNFkyVndkQ0JQVTBWeWNtOXlPbkJoYzNOY2JpTWdRWFIwWlcxd2RDQmpiRzlqYXlCd1pYSnph"
    "WE4wWlc1alpTQkNSVVpQVWtVZ2JXRnlhMlZ5TDNOMFlYUmxMMkoxWkdkbGRDQmpiMjF3WVhKcGMyOXVjeTVjYmlNZ1FTQnRa"
    "WFJoWkdGMFlTQmxjbkp2Y2lCa2IyVnpJRzV2ZENCd2NtVjJaVzUwSUhSb1pTQnZjbWxuYVc1aGJDQmxjM05sYm5ScFlXd2dj"
    "M1J2Y0NCd2NtOTBiMk52YkM1Y2JteGhZajF3WVhSb2JHbGlMbEJoZEdnb1kxc25iR0ZpSjEwcE8yMWhjbXRsY2w5emRHRjBa"
    "VDBuYkdGaVgyRmljMlZ1ZENkY2JuUnllVHBjYmlBZ0lDQnBaaUJzWVdJdVpYaHBjM1J6S0NrNlhHNGdJQ0FnSUNBZ0lHbHVa"
    "bTg5YkdGaUxteHpkR0YwS0NsY2JpQWdJQ0FnSUNBZ2FXWWdibTkwSUhOMFlYUXVVMTlKVTBSSlVpaHBibVp2TG5OMFgyMXZa"
    "R1VwSUc5eUlITjBZWFF1VTE5SlRVOUVSU2hwYm1adkxuTjBYMjF2WkdVcElUMHdiemN3TUNCdmNpQnBibVp2TG5OMFgzVnBa"
    "Q0U5YjNNdVoyVjBkV2xrS0NrNlhHNGdJQ0FnSUNBZ0lDQWdJQ0J0WVhKclpYSmZjM1JoZEdVOUoyOTNibVZ5YzJocGNGOTFi"
    "bXR1YjNkdUoxeHVJQ0FnSUNBZ0lDQmxiSE5sT2x4dUlDQWdJQ0FnSUNBZ0lDQWdiV0Z5YTJWeVgzTjBZWFJsUFNkemRHOXdY"
    "M0psY1hWbGMzUmxaQ2RjYmlBZ0lDQWdJQ0FnSUNBZ0lHWnZjaUJ1WVcxbElHbHVJQ2dvSjNWcExXWmhhV3gxY21VbkxDZHpk"
    "Rzl3SnlrZ2FXWWdZMXNuWm1seWMzUmZabUZwYkhWeVpTZGRJR2x6SUc1dmRDQk9iMjVsSUdWc2MyVWdLQ2R6ZEc5d0p5d3BL"
    "VHBjYmlBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0IwY25rNlhHNGdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJR1prUFc5ekxtOXda"
    "VzRvYkdGaUwyNWhiV1VzYjNNdVQxOVhVazlPVEZsOGIzTXVUMTlEVWtWQlZIeHZjeTVQWDBWWVEweDhiM011VDE5T1QwWlBU"
    "RXhQVnl3d2J6WXdNQ2xjYmlBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ2IzTXVZMnh2YzJVb1ptUXBYRzRnSUNBZ0lDQWdJ"
    "Q0FnSUNBZ0lDQWdaWGhqWlhCMElFWnBiR1ZPYjNSR2IzVnVaRVZ5Y205eU9tMWhjbXRsY2w5emRHRjBaVDBuYkdGaVgyRmlj"
    "MlZ1ZENkY2JpQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNCbGVHTmxjSFFnUm1sc1pVVjRhWE4wYzBWeWNtOXlPbkJoYzNOY2JtVjRZ"
    "MlZ3ZENCUFUwVnljbTl5T20xaGNtdGxjbDl6ZEdGMFpUMG5jM1J2Y0Y5MWJtTnZibVpwY20xbFpDZGNibkJ5YVc1MEtHcHpi"
    "MjR1WkhWdGNITW9leWRqYkc5amF5YzZZMnh2WTJzc0oyVjJaVzUwWDNOMFlYUmxKenBsZG1WdWRGOXpkR0YwWlN3bmJXRnlh"
    "MlZ5WDNOMFlYUmxKenB0WVhKclpYSmZjM1JoZEdWOUtTbGNiaUk3Q21OdmJuTjBJRkpGUVVSQ1FVTkxQU0pwYlhCdmNuUWdh"
    "bk52Yml4dmN5eHdZWFJvYkdsaUxITjBZWFFzYzNWaWNISnZZMlZ6Y3l4emVYTXNkR2x0WlZ4dVl6MXFjMjl1TG14dllXUnpL"
    "SE41Y3k1aGNtZDJXekZkS1Z4dVkyaHBiR1J5Wlc0OVRtOXVaVHRvWld4d1pYSmZjR2xrUFdOYkoyaGxiSEJsY2w5d2FXUW5Y"
    "VHR2ZFhSbGNsOXZZbk5sY25aaGRHbHZiajFPYjI1bE8zQnliMnBsWTNScGIyNDlKM1Z1WVhaaGFXeGhZbXhsSjF4dVpHVm1J"
    "R1pwYm1sMFpWOXZZbk5sY25aaGRHbHZiaWgyWVd4MVpTazZYRzRnSUNBZ2FXWWdkSGx3WlNoMllXeDFaU2tnYVhNZ2JtOTBJ"
    "R1JwWTNRZ2IzSWdjMlYwS0haaGJIVmxLU0U5SUhzbmIySnpaWEoyWldRbkxDZGthV0ZuYm05emRHbGpKMzBnYjNJZ2RIbHda"
    "U2gyWVd4MVpWc25iMkp6WlhKMlpXUW5YU2tnYVhNZ2JtOTBJR0p2YjJ3NlhHNGdJQ0FnSUNBZ0lISmxkSFZ5YmlCT2IyNWxY"
    "RzRnSUNBZ1pHbGhaMjV2YzNScFl6MTJZV3gxWlZzblpHbGhaMjV2YzNScFl5ZGRYRzRnSUNBZ2FXWWdaR2xoWjI1dmMzUnBZ"
    "eUJwY3lCT2IyNWxPbHh1SUNBZ0lDQWdJQ0J5WlhSMWNtNGdleWR2WW5ObGNuWmxaQ2M2ZG1Gc2RXVmJKMjlpYzJWeWRtVmtK"
    "MTBzSjJScFlXZHViM04wYVdNbk9rNXZibVY5WEc0Z0lDQWdhV1lnYm05MElIWmhiSFZsV3lkdlluTmxjblpsWkNkZElHOXlJ"
    "SFI1Y0dVb1pHbGhaMjV2YzNScFl5a2dhWE1nYm05MElHUnBZM1FnYjNJZ2MyVjBLR1JwWVdkdWIzTjBhV01wSVQwZ2V5ZHph"
    "WFJsSnl3blpYaGpaWEIwYVc5dVgyTnNZWE56Snl3bmIzZHVYMloxYm1OMGFXOXVKeXduYjNkdVgyeHBibVVuZlRwY2JpQWdJ"
    "Q0FnSUNBZ2NtVjBkWEp1SUU1dmJtVmNiaUFnSUNCemFYUmxjejBvSjJoaGJtUnNaWEluTENkelpYSjJaWEluTENkdFlXbHVK"
    "eWxjYmlBZ0lDQnJhVzVrY3owb0owRjBkSEpwWW5WMFpVVnljbTl5Snl3blZIbHdaVVZ5Y205eUp5d25WbUZzZFdWRmNuSnZj"
    "aWNzSjB0bGVVVnljbTl5Snl3blQxTkZjbkp2Y2ljc0owSnliMnRsYmxCcGNHVkZjbkp2Y2ljc0owTnZibTVsWTNScGIyNVNa"
    "WE5sZEVWeWNtOXlKeXduVkdsdFpXOTFkRVZ5Y205eUp5d25iM1JvWlhJbktWeHVJQ0FnSUdaMWJtTjBhVzl1Y3owb0owaGxZ"
    "V1JsY2xKbFlXUmxjaTV5WldGa2JHbHVaU2NzSjBSbGJXOVRaWEoyWlhJdWNISnZZMlZ6YzE5eVpYRjFaWE4wSnl3blJHVnRi"
    "eTVpWldkcGJpY3NKMFJsYlc4dVkyRnNiR0poWTJzbkxDZEVaVzF2TG1sdWRtOXJaU2NzSjBoaGJtUnNaWEl1YUdGdVpHeGxY"
    "Mjl1WlY5eVpYRjFaWE4wSnl3blNHRnVaR3hsY2k1elpXNWtYMlZ5Y205eUp5d25TR0Z1Wkd4bGNpNW5aWFFuTENkSVlXNWti"
    "R1Z5TG5KbGNHeDVKeXduYldGcGJpY3BYRzRnSUNBZ2MybDBaU3hyYVc1a0xHWjFibU4wYVc5dUxHeHBibVU5S0dScFlXZHVi"
    "M04wYVdOYmExMGdabTl5SUdzZ2FXNGdLQ2R6YVhSbEp5d25aWGhqWlhCMGFXOXVYMk5zWVhOekp5d25iM2R1WDJaMWJtTjBh"
    "Vzl1Snl3bmIzZHVYMnhwYm1VbktTbGNiaUFnSUNCcFppQjBlWEJsS0hOcGRHVXBJR2x6SUc1dmRDQnpkSElnYjNJZ2MybDBa"
    "U0J1YjNRZ2FXNGdjMmwwWlhNZ2IzSWdkSGx3WlNocmFXNWtLU0JwY3lCdWIzUWdjM1J5SUc5eUlHdHBibVFnYm05MElHbHVJ"
    "R3RwYm1Sek9seHVJQ0FnSUNBZ0lDQnlaWFIxY200Z1RtOXVaVnh1SUNBZ0lHbG1JRzV2ZENBb0tHWjFibU4wYVc5dUlHbHpJ"
    "RTV2Ym1VZ1lXNWtJR3hwYm1VZ2FYTWdUbTl1WlNrZ2IzSWdLSFI1Y0dVb1puVnVZM1JwYjI0cElHbHpJSE4wY2lCaGJtUWda"
    "blZ1WTNScGIyNGdhVzRnWm5WdVkzUnBiMjV6SUdGdVpDQjBlWEJsS0d4cGJtVXBJR2x6SUdsdWRDQmhibVFnTVR3OWJHbHVa"
    "VHc5TVRBeU5Da3BPbHh1SUNBZ0lDQWdJQ0J5WlhSMWNtNGdUbTl1WlZ4dUlDQWdJSEpsZEhWeWJpQjdKMjlpYzJWeWRtVmtK"
    "enBVY25WbExDZGthV0ZuYm05emRHbGpKenA3SjNOcGRHVW5Pbk5wZEdVc0oyVjRZMlZ3ZEdsdmJsOWpiR0Z6Y3ljNmEybHVa"
    "Q3duYjNkdVgyWjFibU4wYVc5dUp6cG1kVzVqZEdsdmJpd25iM2R1WDJ4cGJtVW5PbXhwYm1WOWZWeHViM1YwWlhJOWNHRjBh"
    "R3hwWWk1UVlYUm9LR05iSjI5MWRHVnlYMjkxZENkZEtWeHVhV1lnYjNWMFpYSXVhWE5mWm1sc1pTZ3BPbHh1SUNBZ0lHWmtQ"
    "Vzl6TG05d1pXNG9iM1YwWlhJc2IzTXVUMTlTUkU5T1RGbDhiM011VDE5T1QwWlBURXhQVjN4dmN5NVBYMDVQVGtKTVQwTkxL"
    "Vnh1SUNBZ0lIUnllVHBjYmlBZ0lDQWdJQ0FnYVc1bWJ6MXZjeTVtYzNSaGRDaG1aQ2xjYmlBZ0lDQWdJQ0FnYVdZZ2JtOTBJ"
    "Q2h6ZEdGMExsTmZTVk5TUlVjb2FXNW1ieTV6ZEY5dGIyUmxLU0JoYm1RZ2FXNW1ieTV6ZEY5MWFXUTlQVzl6TG1kbGRIVnBa"
    "Q2dwSUdGdVpDQnpkR0YwTGxOZlNVMVBSRVVvYVc1bWJ5NXpkRjl0YjJSbEtUMDlNRzgyTURBZ1lXNWtJREE4YVc1bWJ5NXpk"
    "Rjl6YVhwbFBEMHlOakl4TkRRcE9seHVJQ0FnSUNBZ0lDQWdJQ0FnY21GcGMyVWdWbUZzZFdWRmNuSnZjaWduWm1sNFpXUmZi"
    "M1YwWlhKZlpYWnBaR1Z1WTJWZmFXNTJZV3hwWkNjcFhHNGdJQ0FnSUNBZ0lIZHBkR2dnYjNNdVptUnZjR1Z1S0daa0xDZHlZ"
    "aWNzWTJ4dmMyVm1aRDFHWVd4elpTa2dZWE1nYzI5MWNtTmxPbkpoZDE5aWVYUmxjejF6YjNWeVkyVXVjbVZoWkNneU5qSXhO"
    "RFVwWEc0Z0lDQWdJQ0FnSUdsbUlHNXZkQ0F3UEd4bGJpaHlZWGRmWW5sMFpYTXBQRDB5TmpJeE5EUTZjbUZwYzJVZ1ZtRnNk"
    "V1ZGY25KdmNpZ25abWw0WldSZmIzVjBaWEpmWlhacFpHVnVZMlZmYVc1MllXeHBaQ2NwWEc0Z0lDQWdJQ0FnSUdSaGRHRTlh"
    "bk52Ymk1c2IyRmtjeWh5WVhkZllubDBaWE1wWEc0Z0lDQWdabWx1WVd4c2VUcHZjeTVqYkc5elpTaG1aQ2xjYmlBZ0lDQnZk"
    "WFJsY2w5dlluTmxjblpoZEdsdmJqMW1hVzVwZEdWZmIySnpaWEoyWVhScGIyNG9aR0YwWVM1blpYUW9KM1Z1Wlhod1pXTjBa"
    "V1JmWm1GcGJIVnlaVjl2WW5ObGNuWmhkR2x2YmljcEtWeHVJQ0FnSUhCeWIycGxZM1JwYjI0OVpHRjBZUzVuWlhRb0ozVnVa"
    "WGh3WldOMFpXUmZiMkp6WlhKMllYUnBiMjVmY0hKdmFtVmpkR2x2YmljcFhHNGdJQ0FnYVdZZ2NISnZhbVZqZEdsdmJpQnVi"
    "M1FnYVc0Z0tDZDFibUYyWVdsc1lXSnNaU2NzSjNaaGJHbGtKeXduYVc1MllXeHBaQ2NwT25CeWIycGxZM1JwYjI0OUoybHVk"
    "bUZzYVdRblhHNGdJQ0FnYVdZZ2NISnZhbVZqZEdsdmJqMDlKM1poYkdsa0p5QmhibVFnYjNWMFpYSmZiMkp6WlhKMllYUnBi"
    "MjRnYVhNZ1RtOXVaVHB3Y205cVpXTjBhVzl1UFNkcGJuWmhiR2xrSjF4dUlDQWdJR0ZzYkc5M1pXUTlleWQzYUc5aGJXa25M"
    "Q2RrYVhOamIzWmxjbmtuTENkamIyNW1hV1JsYm5ScFlXeGZZMnhwWlc1MFgyTnlaV0YwWlNjc0oyOXdaWEpoZEc5eVgyeHZa"
    "Mmx1Snl3bmMyVnlkbVZ5Snl3bmJXRnBiblJsYm1GdVkyVmZhVzVwZENkOVhHNGdJQ0FnYzNSaGNuUmxaRDFrWVhSaExtZGxk"
    "Q2duYUdWc2NHVnlYMmx1ZG05allYUnBiMjV6SnlrOVBURWdZVzVrSUhSNWNHVW9aR0YwWVM1blpYUW9KMmhsYkhCbGNsOXdh"
    "V1FuS1NrZ2FYTWdhVzUwSUdGdVpDQmtZWFJoV3lkb1pXeHdaWEpmY0dsa0oxMCtNRnh1SUNBZ0lHbG1JSE4wWVhKMFpXUTZZ"
    "V3hzYjNkbFpDNWhaR1FvSjJobGJIQmxjaWNwWEc0Z0lDQWdjbUYzUFdSaGRHRXVaMlYwS0NkdmQyNWxaRjlqYUdsc1pGOWxl"
    "R2wwY3ljcFhHNGdJQ0FnYVdZZ2FYTnBibk4wWVc1alpTaHlZWGNzYkdsemRDa2dZVzVrSUd4bGJpaHlZWGNwUFQxc1pXNG9Z"
    "V3hzYjNkbFpDa2dZVzVrSUdGc2JDaHBjMmx1YzNSaGJtTmxLSFlzWkdsamRDa2dZVzVrSUhZdVoyVjBLQ2R1WVcxbEp5a2dh"
    "VzRnWVd4c2IzZGxaQ0JoYm1RZ2RIbHdaU2gyTG1kbGRDZ25jR2xrSnlrcElHbHpJR2x1ZENCaGJtUWdkbHNuY0dsa0oxMCtN"
    "Q0JoYm1RZ2RIbHdaU2gyTG1kbGRDZ25aWGhwZENjcEtTQnBjeUJwYm5RZ1ptOXlJSFlnYVc0Z2NtRjNLU0JoYm1RZ2UzWmJK"
    "MjVoYldVblhTQm1iM0lnZGlCcGJpQnlZWGQ5UFQxaGJHeHZkMlZrSUdGdVpDQnNaVzRvZTNaYkozQnBaQ2RkSUdadmNpQjJJ"
    "R2x1SUhKaGQzMHBQVDFzWlc0b1lXeHNiM2RsWkNrZ1lXNWtJRzVsZUhRb2Rsc25jR2xrSjEwZ1ptOXlJSFlnYVc0Z2NtRjNJ"
    "R2xtSUhaYkoyNWhiV1VuWFQwOUozTmxjblpsY2ljcFBUMWpXeWR6WlhKMlpYSmZjR2xrSjEwZ1lXNWtJQ2dvYzNSaGNuUmxa"
    "Q0JoYm1RZ2JtVjRkQ2gyV3lkd2FXUW5YU0JtYjNJZ2RpQnBiaUJ5WVhjZ2FXWWdkbHNuYm1GdFpTZGRQVDBuYUdWc2NHVnlK"
    "eWs5UFdSaGRHRmJKMmhsYkhCbGNsOXdhV1FuWFNCaGJtUWdLR2hsYkhCbGNsOXdhV1FnYVhNZ1RtOXVaU0J2Y2lCb1pXeHda"
    "WEpmY0dsa1BUMWtZWFJoV3lkb1pXeHdaWEpmY0dsa0oxMHBLU0J2Y2lBb2JtOTBJSE4wWVhKMFpXUWdZVzVrSUdobGJIQmxj"
    "bDl3YVdRZ2FYTWdUbTl1WlNCaGJtUWdaR0YwWVM1blpYUW9KMmhsYkhCbGNsOXBiblp2WTJGMGFXOXVjeWNwSUdseklFNXZi"
    "bVVnWVc1a0lHUmhkR0V1WjJWMEtDZG9aV3h3WlhKZmNHbGtKeWtnYVhNZ1RtOXVaU2twT2x4dUlDQWdJQ0FnSUNCamFHbHNa"
    "SEpsYmoxYmUyczZkbHRyWFNCbWIzSWdheUJwYmlBb0oyNWhiV1VuTENkd2FXUW5MQ2RsZUdsMEp5bDlJR1p2Y2lCMklHbHVJ"
    "SEpoZDExY2JpQWdJQ0FnSUNBZ2FHVnNjR1Z5WDNCcFpEMWtZWFJoV3lkb1pXeHdaWEpmY0dsa0oxMGdhV1lnYzNSaGNuUmxa"
    "Q0JsYkhObElFNXZibVZjYm5CcFpITTliR2x6ZENoald5ZHZkMjVsWkY5d2FXUnpKMTBwWEc1cFppQm9aV3h3WlhKZmNHbGtJ"
    "R2x6SUc1dmRDQk9iMjVsSUdGdVpDQm9aV3h3WlhKZmNHbGtJRzV2ZENCcGJpQndhV1J6T25CcFpITXVZWEJ3Wlc1a0tHaGxi"
    "SEJsY2w5d2FXUXBYRzV3UFhOMVluQnliMk5sYzNNdWNuVnVLRnNuTDJKcGJpOXdjeWNzSnkxd0p5d25MQ2N1YW05cGJpaHpk"
    "SElvZGlrZ1ptOXlJSFlnYVc0Z2NHbGtjeWtzSnkxdkp5d25jR2xrUFNkZExHTmhjSFIxY21WZmIzVjBjSFYwUFZSeWRXVXNk"
    "R2x0Wlc5MWREMHpLVnh1Y0hOZmEyNXZkMjQ5Y0M1eVpYUjFjbTVqYjJSbElHbHVJQ2d3TERFcElHRnVaQ0JoYkd3b2RpNXBj"
    "MlJwWjJsMEtDa2dabTl5SUhZZ2FXNGdjQzV6ZEdSdmRYUXVjM0JzYVhRb0tTbGNibkJ5WlhObGJuUTljMlYwS0dsdWRDaDJL"
    "U0JtYjNJZ2RpQnBiaUJ3TG5OMFpHOTFkQzV6Y0d4cGRDZ3BLU0JwWmlCd2MxOXJibTkzYmlCbGJITmxJSE5sZENncFhHNXdi"
    "M0owY3oxN2ZWeHVabTl5SUhCdmNuUWdhVzRnS0Rrd01EQXNNekF3TUNrNlhHNGdJQ0FnY0QxemRXSndjbTlqWlhOekxuSjFi"
    "aWhiSnk5MWMzSXZjMkpwYmk5c2MyOW1KeXduTFc1UUp5d25MWFFuTENjdGFWUkRVRG9uSzNOMGNpaHdiM0owS1N3bkxYTlVR"
    "MUE2VEVsVFZFVk9KMTBzWTJGd2RIVnlaVjl2ZFhSd2RYUTlWSEoxWlN4MGFXMWxiM1YwUFRNcFhHNGdJQ0FnY0c5eWRITmJj"
    "M1J5S0hCdmNuUXBYVDF1YjNRZ1ltOXZiQ2h3TG5OMFpHOTFkQzV6ZEhKcGNDZ3BLU0JwWmlCd0xuSmxkSFZ5Ym1OdlpHVWdh"
    "VzRnS0RBc01Ta2daV3h6WlNCT2IyNWxYRzV3Y21sdWRDaHFjMjl1TG1SMWJYQnpLSHNuWTJ4dlkyc25PbnNuYlc5dWIzUnZi"
    "bWxqWDI1ekp6cHpkSElvZEdsdFpTNXRiMjV2ZEc5dWFXTmZibk1vS1Nrc0ozZGhiR3hmWlhCdlkyaGZibk1uT25OMGNpaDBh"
    "VzFsTG5ScGJXVmZibk1vS1NsOUxDZHZkMjVsWkY5d2FXUnpKenA3YzNSeUtIWXBPaWgySUc1dmRDQnBiaUJ3Y21WelpXNTBJ"
    "R2xtSUhCelgydHViM2R1SUdWc2MyVWdUbTl1WlNrZ1ptOXlJSFlnYVc0Z2NHbGtjMzBzSjNCdmNuUnpKenB3YjNKMGN5d25i"
    "R0ZpWDJGaWMyVnVkQ2M2Ym05MElIQmhkR2hzYVdJdVVHRjBhQ2hqV3lkc1lXSW5YU2t1WlhocGMzUnpLQ2tzSjI5M2JtVmtY"
    "Mk5vYVd4a1gyVjRhWFJ6SnpwamFHbHNaSEpsYml3bmFHVnNjR1Z5WDNCcFpDYzZhR1ZzY0dWeVgzQnBaQ3duZFc1bGVIQmxZ"
    "M1JsWkY5bVlXbHNkWEpsWDI5aWMyVnlkbUYwYVc5dUp6cHZkWFJsY2w5dlluTmxjblpoZEdsdmJpd25kVzVsZUhCbFkzUmxa"
    "Rjl2WW5ObGNuWmhkR2x2Ymw5d2NtOXFaV04wYVc5dUp6cHdjbTlxWldOMGFXOXVmU2twWEc0aU93cGpiMjV6ZENCTlFWSkxS"
    "Vkk5SW1sdGNHOXlkQ0J2Y3l4d1lYUm9iR2xpTEhOMFlYUXNjM2x6WEc1c1lXSTljR0YwYUd4cFlpNVFZWFJvS0hONWN5NWhj"
    "bWQyV3pGZEtUdG5kV0Z5WkQxcGJuUW9jM2x6TG1GeVozWmJNbDBwTzNObGNuWmxjajFwYm5Rb2MzbHpMbUZ5WjNaYk0xMHBY"
    "RzVwWmlCbmRXRnlaRHc5TUNCdmNpQnpaWEoyWlhJOFBUQWdiM0lnWjNWaGNtUTlQWE5sY25abGNqcHlZV2x6WlNCV1lXeDFa"
    "VVZ5Y205eUtDZHZkMjVsWkY5amIyNTBaWGgwWDJsdWRtRnNhV1FuS1Z4dWFXNW1iejFzWVdJdWJITjBZWFFvS1Z4dWFXWWdi"
    "bTkwSUNoemRHRjBMbE5mU1ZORVNWSW9hVzVtYnk1emRGOXRiMlJsS1NCaGJtUWdjM1JoZEM1VFgwbE5UMFJGS0dsdVptOHVj"
    "M1JmYlc5a1pTazlQVEJ2TnpBd0lHRnVaQ0JwYm1adkxuTjBYM1ZwWkQwOWIzTXVaMlYwZFdsa0tDa3BPbkpoYVhObElGWmhi"
    "SFZsUlhKeWIzSW9KMjkzYm1Wa1gyTnZiblJsZUhSZmFXNTJZV3hwWkNjcFhHNXZjeTVyYVd4c0tHZDFZWEprTERBcE8yOXpM"
    "bXRwYkd3b2MyVnlkbVZ5TERBcFhHNXBaaUFvYkdGaUx5ZHpkRzl3SnlrdVpYaHBjM1J6S0NrZ2IzSWdLR3hoWWk4bmRXa3Ra"
    "bUZwYkhWeVpTY3BMbVY0YVhOMGN5Z3BPbkpoYVhObElGWmhiSFZsUlhKeWIzSW9KMjkzYm1Wa1gyTnZiblJsZUhSZmFXNTJZ"
    "V3hwWkNjcFhHNW1aRDF2Y3k1dmNHVnVLR3hoWWk4blluSnZkM05sY2kxd2NtVndZWEpsWkNjc2IzTXVUMTlYVWs5T1RGbDhi"
    "M011VDE5RFVrVkJWSHh2Y3k1UFgwVllRMHg4YjNNdVQxOU9UMFpQVEV4UFZ5d3diell3TUNsY2JtOXpMbU5zYjNObEtHWmtL"
    "Vnh1Y0hKcGJuUW9KM3RjSW5CeVpYQmhjbVZrWDIxaGNtdGxjbDkzY21sMGRHVnVYQ0k2ZEhKMVpYMG5LVnh1SWpzS1kyOXVj"
    "M1FnVUVWU1UwbFRWRDBpYVcxd2IzSjBJR3B6YjI0c2IzTXNjR0YwYUd4cFlpeHplWE5jYm5CaGRHZzljR0YwYUd4cFlpNVFZ"
    "WFJvS0hONWN5NWhjbWQyV3pGZEtWeHVjbVZqYjNKa1BXcHpiMjR1Ykc5aFpITW9jM2x6TG1GeVozWmJNbDBwWEc0aklFTnZi"
    "blpsY25RZ1pHVmphVzFoYkNCemRISnBibWR6SUdScGNtVmpkR3g1SUhSdklGQjVkR2h2YmlCcGJuUmxaMlZ5Y3l3Z1lYWnZh"
    "V1JwYm1jZ1NsTWdUblZ0WW1WeUlISnZkVzVrYVc1bkxseHVaR1ZtSUdOc2IyTnJjeWgyWVd4MVpTazZYRzRnSUNBZ2FXWWdh"
    "WE5wYm5OMFlXNWpaU2gyWVd4MVpTeGthV04wS1RwY2JpQWdJQ0FnSUNBZ1ptOXlJR3NzZGlCcGJpQnNhWE4wS0haaGJIVmxM"
    "bWwwWlcxektDa3BPbHh1SUNBZ0lDQWdJQ0FnSUNBZ2FXWWdheUJwYmlBb0oyMXZibTkwYjI1cFkxOXVjeWNzSjNkaGJHeGZa"
    "WEJ2WTJoZmJuTW5LU0JoYm1RZ2FYTnBibk4wWVc1alpTaDJMSE4wY2lrNlhHNGdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ1lYTnpa"
    "WEowSUhZdWFYTmtaV05wYldGc0tDa2dZVzVrSUd4bGJpaDJLVHc5TVRrZ1lXNWtJREE4UFdsdWRDaDJLVHd5S2lvMk0xeHVJ"
    "Q0FnSUNBZ0lDQWdJQ0FnSUNBZ0lIWmhiSFZsVzJ0ZFBXbHVkQ2gyS1Z4dUlDQWdJQ0FnSUNBZ0lDQWdaV3h6WlRwamJHOWph"
    "M01vZGlsY2JpQWdJQ0JsYkdsbUlHbHphVzV6ZEdGdVkyVW9kbUZzZFdVc2JHbHpkQ2s2WEc0Z0lDQWdJQ0FnSUdadmNpQjJJ"
    "R2x1SUhaaGJIVmxPbU5zYjJOcmN5aDJLVnh1WTJ4dlkydHpLSEpsWTI5eVpDbGNibVprUFc5ekxtOXdaVzRvY0dGMGFDeHZj"
    "eTVQWDFkU1QwNU1XWHh2Y3k1UFgwTlNSVUZVZkc5ekxrOWZSVmhEVEh4dmN5NVBYMDVQUms5TVRFOVhMREJ2TmpBd0tWeHVk"
    "MmwwYUNCdmN5NW1aRzl3Wlc0b1ptUXNKM2NuTEdWdVkyOWthVzVuUFNkaGMyTnBhU2NwSUdGeklHWTZYRzRnSUNBZ1ppNTNj"
    "bWwwWlNocWMyOXVMbVIxYlhCektISmxZMjl5WkN4emIzSjBYMnRsZVhNOVZISjFaU3hwYm1SbGJuUTlNaWtySjF4Y2JpY3BP"
    "Mll1Wm14MWMyZ29LVHR2Y3k1bWMzbHVZeWhtTG1acGJHVnVieWdwS1Z4dWNISnBiblFvYW5OdmJpNWtkVzF3Y3loN0ozZHlh"
    "WFIwWlc1ZlpYaGpiSFZ6YVhabEp6cFVjblZsZlNrcFhHNGlPd3BqYjI1emRDQnZkMjQ5Ykc5aFpDZ2laREF4WDJsdGJXVmth"
    "V0YwWlY5dmQyNWxaRjlvWVc1a2JHVnpJaWs3Q21OdmJuTjBJRTlDVTB0RldUMGlaREF4WDJsdGJXVmthV0YwWlY5dlluTmxj"
    "blpoZEdsdmJsOXlaV052Y21RaU93cHBaaUFvSVc5M2JpQjhmQ0J2ZDI0dWNuVnVkR2x0WlY5eVpXeGxZWE5sSVQwOWRISjFa"
    "U0I4ZkNCdmQyNHVaSEpwZG1WeVgyOTNibVZrSVQwOWRISjFaU0I4ZkFvZ0lDQWdiM2R1TG1WNFlXTjBYMkp2ZFc1a0lUMDlk"
    "SEoxWlNCOGZDQnZkMjR1YzJsdVoyeGxYMk52Ym5SeWIyeHNaWEloUFQxMGNuVmxJSHg4Q2lBZ0lDQWhXeUp3Y21Wd1lYSmxa"
    "RjlsYm5SeWVTSXNJbUp5YjNkelpYSmZaR1ZqYVhOcGIyNGlYUzVwYm1Oc2RXUmxjeWh2ZDI0dWNHaGhjMlVwSUh4OENpQWdJ"
    "Q0FvYjNkdUxuQm9ZWE5sUFQwOUluQnlaWEJoY21Wa1gyVnVkSEo1SWlZbUtHOTNiaTV3Y21Wd1lYSmxaRjluWVhSbElUMDlk"
    "SEoxWlh4OGIzZHVMbWhsYkhCbGNsOXdhV1FoUFQxdWRXeHNmSHdLSUNBZ0lDQWdiM2R1TG1acGVIUjFjbVZmY21WaFpIazlQ"
    "VDEwY25WbGZIeHZkMjR1YkdsemRHVnVaWEpmY0dsa1gzQnliMjltUFQwOWRISjFaU2twSUh4OENpQWdJQ0FvYjNkdUxuQm9Z"
    "WE5sUFQwOUltSnliM2R6WlhKZlpHVmphWE5wYjI0aUppWW9iM2R1TG1acGVIUjFjbVZmY21WaFpIa2hQVDEwY25WbGZIeHZk"
    "MjR1YkdsemRHVnVaWEpmY0dsa1gzQnliMjltSVQwOWRISjFaU2twSUh4OENpQWdJQ0J2ZDI0dVpuSmxjMmhmY0dGMGFITmZj"
    "SEpsWm14cFoyaDBJVDA5ZEhKMVpTQjhmQW9nSUNBZ0lVNTFiV0psY2k1cGMxTmhabVZKYm5SbFoyVnlLRzkzYmk1emRHRnlk"
    "RjlsY0c5amFGOXRjeWtnZkh3Z2RIbHdaVzltSUc5M2JpNTNiM0pyYzNCaFkyVWhQVDBpYzNSeWFXNW5JaUI4ZkFvZ0lDQWdi"
    "bVYzSUZObGRDaGJiM2R1TG1kMVlYSmtYM0JwWkN4dmQyNHVjMlZ5ZG1WeVgzQnBaQ3h2ZDI0dVluSnZkM05sY2w5d2FXUXND"
    "aUFnSUNBZ0lDQXVMaTRvYjNkdUxtaGxiSEJsY2w5d2FXUTlQVDF1ZFd4c1AxdGRPbHR2ZDI0dWFHVnNjR1Z5WDNCcFpGMHBY"
    "U2t1YzJsNlpTRTlQU2h2ZDI0dWFHVnNjR1Z5WDNCcFpEMDlQVzUxYkd3L016bzBLU0I4ZkFvZ0lDQWdJVnR2ZDI0dVozVmhj"
    "bVJmY0dsa0xHOTNiaTV6WlhKMlpYSmZjR2xrTEc5M2JpNWljbTkzYzJWeVgzQnBaQ3h2ZDI0dVpYaGxZMTl6WlhOemFXOXVM"
    "QW9nSUNBZ0lDQWdMaTR1S0c5M2JpNW9aV3h3WlhKZmNHbGtQVDA5Ym5Wc2JEOWJYVHBiYjNkdUxtaGxiSEJsY2w5d2FXUmRL"
    "VjBLSUNBZ0lDQWdJQzVsZG1WeWVTaDJQVDVPZFcxaVpYSXVhWE5UWVdabFNXNTBaV2RsY2loMktTWW1kajR3S1NCOGZBb2dJ"
    "Q0FnSVZ0dmQyNHVjMlZ6YzJsdmJpeHZkMjR1ZEdGeVoyVjBYMmxrTEc5M2JpNTBZV0pmYVdRc2IzZHVMbXhoWWl4dmQyNHVi"
    "M1YwWlhKZmIzVjBMQW9nSUNBZ0lDQWdiM2R1TG1WMlpXNTBYMjkxZEN4dmQyNHVZMnhsWVc1MWNGOXZkWFJkTG1WMlpYSjVL"
    "SFk5UG5SNWNHVnZaaUIyUFQwOUluTjBjbWx1WnlJbUpuWXViR1Z1WjNSb1BqQXBLU0I3Q2lBZ2RHVjRkQ2g3Y0hKdmNHOXpZ"
    "V3hmY21WbWRYTmxaRG9pWm5KbGMyaGZiM2R1WldSZlkyOXVkR1Y0ZEY5eVpYRjFhWEpsWkNKOUtUdGxlR2wwS0NrN0NuMEtZ"
    "Mjl1YzNRZ2NISnBiM0k5Ykc5aFpDaFBRbE5MUlZrcE93cHBaaWh2ZDI0dVkyeHZjMlZrUFQwOWRISjFaWHg4Y0hKcGIzSS9M"
    "bU5zWldGdWRYQmZjM1JoY25SbFpEMDlQWFJ5ZFdVcGV3b2dJSFJsZUhRb2UzQnliM0J2YzJGc1gzSmxablZ6WldRNkltWnBl"
    "SFIxY21WZllXeHlaV0ZrZVY5emRHOXdjR1ZrSWl4eVpYTnZkWEpqWlY5eVpXeGxZWE5sWDNCeWIzWmxianBtWVd4elpYMHBP"
    "MlY0YVhRb0tUc0tmUXBqYjI1emRDQnlaV052Y21ROWNISnBiM0kvUDNzS0lDQnpZMmhsYldFNkluSnBZWFYwYUM1a01ERXRh"
    "VzF0WldScFlYUmxMVzlpYzJWeWRtRjBhVzl1TFdOc1pXRnVkWEF2ZGpFaUxBb2dJR1pwY25OMFgyWmhhV3gxY21VNmJuVnNi"
    "Q3htYVhKemRGOXZZbk5sY25aaGRHbHZibDkzWVd4c1gyMXpPbTUxYkd3c0NpQWdabWx5YzNSZlpYWmxiblJmY0hKdmRtVnVP"
    "bVpoYkhObExIZG9iMnhsWDJOc1pXRnVkWEJmZDJsMGFHbHVOakJmY0hKdmRtVnVPbVpoYkhObExBb2dJSFZ1Wlhod1pXTjBa"
    "V1JmWm1GcGJIVnlaVjl2WW5ObGNuWmhkR2x2YmpwdWRXeHNMR1JwWVdkdWIzTjBhV05mWlhKeWIzSnpPbHRkTEdOc1pXRnVk"
    "WEJmYzNSaGNuUmxaRHBtWVd4elpTd0tJQ0JtYVhKemRGOWpiRzlqYXpwdWRXeHNMRzlpYzJWeWRtRjBhVzl1WDNKbFkyVnBj"
    "SFJ6T2x0ZExHTnZiblJ5YjJ4c1pYSmZiMkp6WlhKMllYUnBiMjV6T2x0ZExHeHZZMkZzWDNSdmIyeGZjbVZqWldsd2RITTZX"
    "MTBzWVdOMGFXOXVjenBiWFN4amJHVmhiblZ3WDJWeWNtOXljenBiWFN3S0lDQm1hVzVoYkY5aFluTmxibU5sT201MWJHd3Ni"
    "M2R1WldSZlkyaHBiR1JmWlhocGRITTZiblZzYkN4amIyNTBjbTlzYkdWeVgyVjRhWFE2Ym5Wc2JDd0tJQ0JtYVhKemRGOXZZ"
    "bk5sY25aaGRHbHZibDkwYjE5bWFXNWhiRjkzWVd4c1gyMXpPbTUxYkd3c2JHRjBZMmhmZEc5ZllXSnpaVzVqWlY5dGN6cHVk"
    "V3hzQ24wN0NtTnZibk4wSUhOeFBYTTlQaUluSWl0VGRISnBibWNvY3lrdWNtVndiR0ZqWlNndkp5OW5MQ0luWEZ3bkp5SXBL"
    "eUluSWpzS1kyOXVjM1FnY21WMFlXbHVQU2dwUFQ1emRHOXlaU2hQUWxOTFJWa3NjbVZqYjNKa0tUc0tZMjl1YzNRZ1kyeGxZ"
    "VzUxY0VWeWNtOXlQV3hoWW1Wc1BUNTdDaUFnYVdZb0lYSmxZMjl5WkM1amJHVmhiblZ3WDJWeWNtOXljeTVwYm1Oc2RXUmxj"
    "eWhzWVdKbGJDa3BjbVZqYjNKa0xtTnNaV0Z1ZFhCZlpYSnliM0p6TG5CMWMyZ29iR0ZpWld3cE93b2dJSEpsZEdGcGJpZ3BP"
    "d3A5T3dwamIyNXpkQ0JzYjJOaGJEMWhjM2x1WXloemIzVnlZMlVzWVhKbktUMCtld29nSUdOdmJuTjBJR050WkQwaWNIbDBh"
    "Rzl1TXlBdFl5QWlLM054S0hOdmRYSmpaU2tyS0dGeVp6MDlQWFZ1WkdWbWFXNWxaRDhpSWpvaUlDSXJjM0VvU2xOUFRpNXpk"
    "SEpwYm1kcFpua29ZWEpuS1NrcE93b2dJR052Ym5OMElISTlZWGRoYVhRZ2RHOXZiSE11WlhobFkxOWpiMjF0WVc1a0tIdGpi"
    "V1FzZDI5eWEyUnBjanB2ZDI0dWQyOXlhM053WVdObExBb2dJQ0FnZVdsbGJHUmZkR2x0WlY5dGN6b3hNREF3TUN4dFlYaGZi"
    "M1YwY0hWMFgzUnZhMlZ1Y3pveU1EQXdmU2s3Q2lBZ2NtVmpiM0prTG14dlkyRnNYM1J2YjJ4ZmNtVmpaV2x3ZEhNdWNIVnph"
    "Q2g3Q2lBZ0lDQnNZV0psYkRwemIzVnlZMlU5UFQxRFRFOURTejhpWTJ4dlkyc2lPbk52ZFhKalpUMDlQVk5VVDFBL0luTjBi"
    "M0FpT25OdmRYSmpaVDA5UFZKRlFVUkNRVU5MUHlKeVpXRmtZbUZqYXlJNkluVnVhMjV2ZDI0aUxBb2dJQ0FnWlhocGREcDBl"
    "WEJsYjJZZ2NpNWxlR2wwWDJOdlpHVTlQVDBpYm5WdFltVnlJajl5TG1WNGFYUmZZMjlrWlRwdWRXeHNMQW9nSUNBZ2MyVnpj"
    "Mmx2Ymw5cFpEcDBlWEJsYjJZZ2NpNXpaWE56YVc5dVgybGtQVDA5SW01MWJXSmxjaUkvY2k1elpYTnphVzl1WDJsa09tNTFi"
    "R3dLSUNCOUtUdHlaWFJoYVc0b0tUc2dMeThnVG5WdFpYSnBZeUJqYjJ4c1pXTjBiM0lnY21WalpXbHdkQ0JDUlVaUFVrVWdZ"
    "Mjl0Y0dGeWFYTnZibk11Q2lBZ2FXWW9jaTV6WlhOemFXOXVYMmxrSVQwOWRXNWtaV1pwYm1Wa0tTQjdDaUFnSUNCamJHVmhi"
    "blZ3UlhKeWIzSW9JbTkzYm1Wa1gyeHZZMkZzWDJOdmJXMWhibVJmZFc1cWIybHVaV1FpS1R0MGFISnZkeUJ1WlhjZ1JYSnli"
    "M0lvSW14dlkyRnNYM0JsYm1ScGJtY2lLVHNLSUNCOUNpQWdhV1lvY2k1bGVHbDBYMk52WkdVaFBUMHdLWFJvY205M0lHNWxk"
    "eUJGY25KdmNpZ2liRzlqWVd4ZlptRnBiR1ZrSWlrN0NpQWdjbVYwZFhKdUlFcFRUMDR1Y0dGeWMyVW9jaTV2ZFhSd2RYUXBP"
    "d3A5T3dwbWRXNWpkR2x2YmlCbWFXNXBkR1ZQWW5ObGNuWmhkR2x2YmloMllXeDFaU2tnZXdvZ0lHTnZibk4wSUdWNFlXTjBQ"
    "U2gyTEd0bGVYTXBQVDUySVQwOWJuVnNiQ1ltZEhsd1pXOW1JSFk5UFQwaWIySnFaV04wSWlZbUlVRnljbUY1TG1selFYSnlZ"
    "WGtvZGlrbUpnb2dJQ0FnVDJKcVpXTjBMbXRsZVhNb2Rpa3ViR1Z1WjNSb1BUMDlhMlY1Y3k1c1pXNW5kR2dtSm10bGVYTXVa"
    "WFpsY25rb2F6MCtUMkpxWldOMExtaGhjMDkzYmloMkxHc3BLVHNLSUNCcFppZ2haWGhoWTNRb2RtRnNkV1VzV3lKdlluTmxj"
    "blpsWkNJc0ltUnBZV2R1YjNOMGFXTWlYU2w4ZkhSNWNHVnZaaUIyWVd4MVpTNXZZbk5sY25abFpDRTlQU0ppYjI5c1pXRnVJ"
    "aWx5WlhSMWNtNGdiblZzYkRzS0lDQmpiMjV6ZENCa1BYWmhiSFZsTG1ScFlXZHViM04wYVdNN0NpQWdhV1lvWkQwOVBXNTFi"
    "R3dwY21WMGRYSnVJSHR2WW5ObGNuWmxaRHAyWVd4MVpTNXZZbk5sY25abFpDeGthV0ZuYm05emRHbGpPbTUxYkd4OU93b2dJ"
    "R2xtS0NGMllXeDFaUzV2WW5ObGNuWmxaSHg4SVdWNFlXTjBLR1FzV3lKemFYUmxJaXdpWlhoalpYQjBhVzl1WDJOc1lYTnpJ"
    "aXdpYjNkdVgyWjFibU4wYVc5dUlpd2liM2R1WDJ4cGJtVWlYU2w4ZkFvZ0lDQWdJQ0ZiSW1oaGJtUnNaWElpTENKelpYSjJa"
    "WElpTENKdFlXbHVJbDB1YVc1amJIVmtaWE1vWkM1emFYUmxLWHg4Q2lBZ0lDQWdJVnNpUVhSMGNtbGlkWFJsUlhKeWIzSWlM"
    "Q0pVZVhCbFJYSnliM0lpTENKV1lXeDFaVVZ5Y205eUlpd2lTMlY1UlhKeWIzSWlMQ0pQVTBWeWNtOXlJaXdpUW5KdmEyVnVV"
    "R2x3WlVWeWNtOXlJaXdLSUNBZ0lDQWdJQ0pEYjI1dVpXTjBhVzl1VW1WelpYUkZjbkp2Y2lJc0lsUnBiV1Z2ZFhSRmNuSnZj"
    "aUlzSW05MGFHVnlJbDB1YVc1amJIVmtaWE1vWkM1bGVHTmxjSFJwYjI1ZlkyeGhjM01wS1hKbGRIVnliaUJ1ZFd4c093b2dJ"
    "R052Ym5OMElHWjFibU4wYVc5dWN6MWJJa2hsWVdSbGNsSmxZV1JsY2k1eVpXRmtiR2x1WlNJc0lrUmxiVzlUWlhKMlpYSXVj"
    "SEp2WTJWemMxOXlaWEYxWlhOMElpd2lSR1Z0Ynk1aVpXZHBiaUlzSWtSbGJXOHVZMkZzYkdKaFkyc2lMQW9nSUNBZ0lrUmxi"
    "Vzh1YVc1MmIydGxJaXdpU0dGdVpHeGxjaTVvWVc1a2JHVmZiMjVsWDNKbGNYVmxjM1FpTENKSVlXNWtiR1Z5TG5ObGJtUmZa"
    "WEp5YjNJaUxDSklZVzVrYkdWeUxtZGxkQ0lzSWtoaGJtUnNaWEl1Y21Wd2JIa2lMQ0p0WVdsdUlsMDdDaUFnYVdZb0lTZ29a"
    "QzV2ZDI1ZlpuVnVZM1JwYjI0OVBUMXVkV3hzSmlaa0xtOTNibDlzYVc1bFBUMDliblZzYkNsOGZBb2dJQ0FnSUNBZ0tHWjFi"
    "bU4wYVc5dWN5NXBibU5zZFdSbGN5aGtMbTkzYmw5bWRXNWpkR2x2YmlrbUprNTFiV0psY2k1cGMxTmhabVZKYm5SbFoyVnlL"
    "R1F1YjNkdVgyeHBibVVwSmlZS0lDQWdJQ0FnSUNCa0xtOTNibDlzYVc1bFBqMHhKaVprTG05M2JsOXNhVzVsUEQweE1ESTBL"
    "U2twY21WMGRYSnVJRzUxYkd3N0NpQWdjbVYwZFhKdUlIdHZZbk5sY25abFpEcDBjblZsTEdScFlXZHViM04wYVdNNmUzTnBk"
    "R1U2WkM1emFYUmxMR1Y0WTJWd2RHbHZibDlqYkdGemN6cGtMbVY0WTJWd2RHbHZibDlqYkdGemN5d0tJQ0FnSUc5M2JsOW1k"
    "VzVqZEdsdmJqcGtMbTkzYmw5bWRXNWpkR2x2Yml4dmQyNWZiR2x1WlRwa0xtOTNibDlzYVc1bGZYMDdDbjBLWm5WdVkzUnBi"
    "MjRnWkdsaFoyNXZjM1JwWXloMllXeDFaU3h3Y205cVpXTjBhVzl1S1NCN0NpQWdZMjl1YzNRZ2NISnZhbVZqZEdWa1BXWnBi"
    "bWwwWlU5aWMyVnlkbUYwYVc5dUtIWmhiSFZsS1RzS0lDQnBaaWh3Y205cVpXTjBhVzl1UFQwOUltbHVkbUZzYVdRaWZId2hX"
    "eUoyWVd4cFpDSXNJblZ1WVhaaGFXeGhZbXhsSWwwdWFXNWpiSFZrWlhNb2NISnZhbVZqZEdsdmJpbDhmQW9nSUNBZ0lDaHdj"
    "bTlxWldOMGFXOXVQVDA5SW5aaGJHbGtJaVltY0hKdmFtVmpkR1ZrUFQwOWJuVnNiQ2twSUhzS0lDQWdJR2xtS0NGeVpXTnZj"
    "bVF1WkdsaFoyNXZjM1JwWTE5bGNuSnZjbk11YVc1amJIVmtaWE1vSW05aWMyVnlkbVZ5WDNCeWIycGxZM1JwYjI1ZmFXNTJZ"
    "V3hwWkNJcEtRb2dJQ0FnSUNCeVpXTnZjbVF1WkdsaFoyNXZjM1JwWTE5bGNuSnZjbk11Y0hWemFDZ2liMkp6WlhKMlpYSmZj"
    "SEp2YW1WamRHbHZibDlwYm5aaGJHbGtJaWs3Q2lBZ2ZRb2dJR2xtS0hCeWIycGxZM1JsWkNFOVBXNTFiR3dtSm5KbFkyOXla"
    "QzUxYm1WNGNHVmpkR1ZrWDJaaGFXeDFjbVZmYjJKelpYSjJZWFJwYjI0OVBUMXVkV3hzS1FvZ0lDQWdjbVZqYjNKa0xuVnVa"
    "WGh3WldOMFpXUmZabUZwYkhWeVpWOXZZbk5sY25aaGRHbHZiajF3Y205cVpXTjBaV1E3Q2lBZ2NtVjBZV2x1S0NrN0lDOHZJ"
    "RTV2SUhKaGR5QmthV0ZuYm05emRHbGpJR1poYkd4aVlXTnJJR0Z1WkNCdWJ5Qm1hWEp6ZEMxbVlXbHNkWEpsSUc5eUlHOTFk"
    "R052YldVZ2JYVjBZWFJwYjI0dUNuMEtZMjl1YzNRZ2JuTTlZejArUW1sblNXNTBLR011Ylc5dWIzUnZibWxqWDI1ektUc0tZ"
    "Mjl1YzNRZ1oyVjBRMnh2WTJzOUtDazlQbXh2WTJGc0tFTk1UME5MS1RzS2JHVjBJR052Ym5SeWIyeHNaWEpLYjJsdVpXUTla"
    "bUZzYzJVN0NteGxkQ0JqYjI1MGNtOXNiR1Z5VUc5c2JFRjJZV2xzWVdKc1pUMTBjblZsT3dwc1pYUWdZMjl1ZEhKdmJHeGxj"
    "a0oxWm1abGNqMGlJanNLYkdWMElHTnNaV0Z1ZFhCVGRHRnlkR1ZrUFdaaGJITmxPd3BzWlhRZ2JHRnpkRk51WVhCemFHOTBS"
    "bXhoWjNNOWJuVnNiRHNLWm5WdVkzUnBiMjRnYkdGMFkyZ29iR0ZpWld3c2NtVmpaV2wyWldSWFlXeHNLU0I3Q2lBZ2FXWW9j"
    "bVZqYjNKa0xtWnBjbk4wWDJaaGFXeDFjbVU5UFQxdWRXeHNLU0I3Q2lBZ0lDQnlaV052Y21RdVptbHljM1JmWm1GcGJIVnla"
    "VDFzWVdKbGJEc0tJQ0FnSUhKbFkyOXlaQzVtYVhKemRGOXZZbk5sY25aaGRHbHZibDkzWVd4c1gyMXpQWEpsWTJWcGRtVmtW"
    "MkZzYkRzS0lDQWdJSEpsZEdGcGJpZ3BPeUF2THlCVGVXNWphSEp2Ym05MWN5Qm1hWEp6ZEMxbVlXbHNkWEpsTDNkaGJHd2di"
    "R0YwWTJnZ1FrVkdUMUpGSUdGdWVTQnVaWGNnWVhkaGFYUXZiM1YwY0hWMExnb2dJSDBLZlFwbWRXNWpkR2x2YmlCdlluTmxj"
    "blpsUTI5dWRISnZiR3hsY2loeUxISmxZMlZwZG1Wa1YyRnNiQ2tnZXdvZ0lHTnZibk4wSUc5aWMyVnlkbUYwYVc5dVBYdHla"
    "V05sYVhabFpGOTNZV3hzWDIxek9uSmxZMlZwZG1Wa1YyRnNiQ3dLSUNBZ0lHVjRhWFE2ZEhsd1pXOW1JSEl1WlhocGRGOWpi"
    "MlJsUFQwOUltNTFiV0psY2lJL2NpNWxlR2wwWDJOdlpHVTZiblZzYkN3S0lDQWdJR2hsYkhCbGNsOWpiMjF3YkdWMFpXUTZa"
    "bUZzYzJVc2FHVnNjR1Z5WDJWNGFYUTZiblZzYkN4bWFYaDBkWEpsWDJacGJtbHphR1ZrT21aaGJITmxmVHNLSUNBdkx5QkRi"
    "MjF3YkdWMFpTQnVkVzFsY21saklISmxjM1ZzZENCd2NtOXFaV04wYVc5dUlHbHpJSEpsZEdGcGJtVmtJRUpGUms5U1JTQmpi"
    "MjF3WVhKcGMyOXVjeTRLSUNCeVpXTnZjbVF1WTI5dWRISnZiR3hsY2w5dlluTmxjblpoZEdsdmJuTXVjSFZ6YUNodlluTmxj"
    "blpoZEdsdmJpazdjbVYwWVdsdUtDazdDaUFnYVdZb2RIbHdaVzltSUhJdVpYaHBkRjlqYjJSbFBUMDlJbTUxYldKbGNpSXBJ"
    "SHNLSUNBZ0lHTnZiblJ5YjJ4c1pYSktiMmx1WldROWRISjFaVHR5WldOdmNtUXVZMjl1ZEhKdmJHeGxjbDlsZUdsMFBYSXVa"
    "WGhwZEY5amIyUmxPM0psZEdGcGJpZ3BPd29nSUgwS0lDQmpiMjUwY205c2JHVnlRblZtWm1WeUt6MTBlWEJsYjJZZ2NpNXZk"
    "WFJ3ZFhROVBUMGljM1J5YVc1bklqOXlMbTkxZEhCMWREb2lJanNLSUNCcFppaGpiMjUwY205c2JHVnlRblZtWm1WeUxteGxi"
    "bWQwYUQ0eE5qTTROQ2tnZXdvZ0lDQWdiR0YwWTJnb0ltTnZiblJ5YjJ4c1pYSmZiMkp6WlhKMllYUnBiMjVmYVc1MllXeHBa"
    "Q0lzY21WalpXbDJaV1JYWVd4c0tUdGpiMjUwY205c2JHVnlRblZtWm1WeVBTSWlPM0psZEhWeWJqc0tJQ0I5Q2lBZ2JHVjBJ"
    "R04xZERzS0lDQjNhR2xzWlNnb1kzVjBQV052Ym5SeWIyeHNaWEpDZFdabVpYSXVhVzVrWlhoUFppZ2lYRzRpS1NrK1BUQXBJ"
    "SHNLSUNBZ0lHTnZibk4wSUd4cGJtVTlZMjl1ZEhKdmJHeGxja0oxWm1abGNpNXpiR2xqWlNnd0xHTjFkQ2s3WTI5dWRISnZi"
    "R3hsY2tKMVptWmxjajFqYjI1MGNtOXNiR1Z5UW5WbVptVnlMbk5zYVdObEtHTjFkQ3N4S1RzS0lDQWdJR2xtS0NGc2FXNWxM"
    "blJ5YVcwb0tTbGpiMjUwYVc1MVpUc0tJQ0FnSUd4bGRDQmxkbVZ1ZERzS0lDQWdJSFJ5ZVh0bGRtVnVkRDFLVTA5T0xuQmhj"
    "bk5sS0d4cGJtVXBPMzFqWVhSamFIc0tJQ0FnSUNBZ2JHRjBZMmdvSW1OdmJuUnliMnhzWlhKZmIySnpaWEoyWVhScGIyNWZh"
    "VzUyWVd4cFpDSXNjbVZqWldsMlpXUlhZV3hzS1R0amIyNTBhVzUxWlRzS0lDQWdJSDBLSUNBZ0lHbG1LRTlpYW1WamRDNW9Z"
    "WE5QZDI0b1pYWmxiblFzSW5WdVpYaHdaV04wWldSZlptRnBiSFZ5WlY5dlluTmxjblpoZEdsdmJpSXBLUW9nSUNBZ0lDQmth"
    "V0ZuYm05emRHbGpLR1YyWlc1MExuVnVaWGh3WldOMFpXUmZabUZwYkhWeVpWOXZZbk5sY25aaGRHbHZiaXhsZG1WdWRDNTFi"
    "bVY0Y0dWamRHVmtYMjlpYzJWeWRtRjBhVzl1WDNCeWIycGxZM1JwYjI0cE93b2dJQ0FnYVdZb1pYWmxiblF1Wm1sNGRIVnla"
    "Vjl5WldGa2VUMDlQWFJ5ZFdVcElIc0tJQ0FnSUNBZ2FXWW9aWFpsYm5RdVozVmhjbVJmY0dsa0lUMDliM2R1TG1kMVlYSmtY"
    "M0JwWkh4OFpYWmxiblF1YzJWeWRtVnlYM0JwWkNFOVBXOTNiaTV6WlhKMlpYSmZjR2xrZkh3S0lDQWdJQ0FnSUNBZ1pYWmxi"
    "blF1YkdGaUlUMDliM2R1TG14aFlueDhJVTUxYldKbGNpNXBjMU5oWm1WSmJuUmxaMlZ5S0dWMlpXNTBMbWhsYkhCbGNsOXdh"
    "V1FwZkh4bGRtVnVkQzVvWld4d1pYSmZjR2xrUEQwd2ZId0tJQ0FnSUNBZ0lDQWdXMjkzYmk1bmRXRnlaRjl3YVdRc2IzZHVM"
    "bk5sY25abGNsOXdhV1FzYjNkdUxtSnliM2R6WlhKZmNHbGtYUzVwYm1Oc2RXUmxjeWhsZG1WdWRDNW9aV3h3WlhKZmNHbGtL"
    "WHg4Q2lBZ0lDQWdJQ0FnSUNodmQyNHVhR1ZzY0dWeVgzQnBaQ0U5UFc1MWJHd21KbTkzYmk1b1pXeHdaWEpmY0dsa0lUMDla"
    "WFpsYm5RdWFHVnNjR1Z5WDNCcFpDa3BDaUFnSUNBZ0lDQWdiR0YwWTJnb0ltWnBlSFIxY21WZmNtVmhaSGxmZFc1amIyNW1h"
    "WEp0WldRaUxISmxZMlZwZG1Wa1YyRnNiQ2s3Q2lBZ0lDQWdJR1ZzYzJVZ2V3b2dJQ0FnSUNBZ0lHOTNiaTVvWld4d1pYSmZj"
    "R2xrUFdWMlpXNTBMbWhsYkhCbGNsOXdhV1E3YjNkdUxtWnBlSFIxY21WZmNtVmhaSGs5ZEhKMVpUdHZkMjR1YkdsemRHVnVa"
    "WEpmY0dsa1gzQnliMjltUFhSeWRXVTdDaUFnSUNBZ0lDQWdjM1J2Y21Vb0ltUXdNVjlwYlcxbFpHbGhkR1ZmYjNkdVpXUmZh"
    "R0Z1Wkd4bGN5SXNiM2R1S1RzS0lDQWdJQ0FnZlFvZ0lDQWdmUW9nSUNBZ2FXWW9aWFpsYm5RdWFHVnNjR1Z5WDJOdmJYQnNa"
    "WFJsWkQwOVBYUnlkV1VwSUhzS0lDQWdJQ0FnYjJKelpYSjJZWFJwYjI0dWFHVnNjR1Z5WDJOdmJYQnNaWFJsWkQxMGNuVmxP"
    "d29nSUNBZ0lDQnZZbk5sY25aaGRHbHZiaTVvWld4d1pYSmZaWGhwZEQxMGVYQmxiMllnWlhabGJuUXVaWGhwZEQwOVBTSnVk"
    "VzFpWlhJaVAyVjJaVzUwTG1WNGFYUTZiblZzYkRzS0lDQWdJQ0FnY21WMFlXbHVLQ2s3Q2lBZ0lDQWdJR2xtS0dWMlpXNTBM"
    "bVY0YVhRaFBUMHdLV3hoZEdOb0tDSm9aV3h3WlhKZlptRnBiR1ZrSWl4eVpXTmxhWFpsWkZkaGJHd3BPd29nSUNBZ0lDQmxi"
    "SE5sSUdsbUtDRmpiR1ZoYm5Wd1UzUmhjblJsWkNZbWNtVmpiM0prTG5CeWIzUmxZM1JsWkY5aFpuUmxjbDl3WVdkbElUMDlk"
    "SEoxWlNrS0lDQWdJQ0FnSUNCc1lYUmphQ2dpYUdWc2NHVnlYMk52YlhCc1pYUmxaRjlpWldadmNtVmZZWEJ3WDJOb1pXTnJj"
    "RzlwYm5RaUxISmxZMlZwZG1Wa1YyRnNiQ2s3Q2lBZ0lDQjlDaUFnSUNCcFppaGxkbVZ1ZEM1bWFYaDBkWEpsWDJacGJtbHph"
    "R1ZrUFQwOWRISjFaU2tnZXdvZ0lDQWdJQ0J2WW5ObGNuWmhkR2x2Ymk1bWFYaDBkWEpsWDJacGJtbHphR1ZrUFhSeWRXVTdj"
    "bVYwWVdsdUtDazdDaUFnSUNBZ0lHbG1LQ0ZqYkdWaGJuVndVM1JoY25SbFpDbHNZWFJqYUNnaVkyOXVkSEp2Ykd4bGNsOWpi"
    "MjF3YkdWMFpXUmZZbVZtYjNKbFgyRndjRjlqYUdWamEzQnZhVzUwSWl4eVpXTmxhWFpsWkZkaGJHd3BPd29nSUNBZ2ZRb2dJ"
    "SDBLSUNCcFppaGpiMjUwY205c2JHVnlTbTlwYm1Wa0ppWWhZMnhsWVc1MWNGTjBZWEowWldRcENpQWdJQ0JzWVhSamFDZ2lZ"
    "Mjl1ZEhKdmJHeGxjbDlqYjIxd2JHVjBaV1JmWW1WbWIzSmxYMkZ3Y0Y5amFHVmphM0J2YVc1MElpeHlaV05sYVhabFpGZGhi"
    "R3dwT3dwOUNtRnplVzVqSUdaMWJtTjBhVzl1SUhCdmJHeERiMjUwY205c2JHVnlLQ2tnZXdvZ0lHbG1LR052Ym5SeWIyeHNa"
    "WEpLYjJsdVpXUjhmQ0ZqYjI1MGNtOXNiR1Z5VUc5c2JFRjJZV2xzWVdKc1pTbHlaWFIxY200N0NpQWdkSEo1SUhzS0lDQWdJ"
    "R052Ym5OMElISTlZWGRoYVhRZ2RHOXZiSE11ZDNKcGRHVmZjM1JrYVc0b2UzTmxjM05wYjI1ZmFXUTZiM2R1TG1WNFpXTmZj"
    "MlZ6YzJsdmJpd0tJQ0FnSUNBZ1kyaGhjbk02SWlJc2VXbGxiR1JmZEdsdFpWOXRjem8xTURBd0xHMWhlRjl2ZFhSd2RYUmZk"
    "RzlyWlc1ek9qSXdNREI5S1RzS0lDQWdJR052Ym5OMElISmxZMlZwZG1Wa1YyRnNiRDFFWVhSbExtNXZkeWdwT3dvZ0lDQWdi"
    "Mkp6WlhKMlpVTnZiblJ5YjJ4c1pYSW9jaXh5WldObGFYWmxaRmRoYkd3cE93b2dJSDBnWTJGMFkyZ2dld29nSUNBZ1kyOXVk"
    "SEp2Ykd4bGNsQnZiR3hCZG1GcGJHRmliR1U5Wm1Gc2MyVTdDaUFnSUNCc1lYUmphQ2dpWTI5dWRISnZiR3hsY2w5dlluTmxj"
    "blpoZEdsdmJsOTFibUYyWVdsc1lXSnNaU0lzUkdGMFpTNXViM2NvS1NrN0NpQWdJQ0JwWmloamJHVmhiblZ3VTNSaGNuUmxa"
    "Q2xqYkdWaGJuVndSWEp5YjNJb0ltTnZiblJ5YjJ4c1pYSmZhbTlwYmw5MWJtTnZibVpwY20xbFpDSXBPd29nSUgwS2ZRcGhj"
    "M2x1WXlCbWRXNWpkR2x2YmlCMGFXMWxaRVJ5YVhabGNpaHNZV0psYkN4aGJHeHZZMkYwYVc5dUxHOXdaWEpoZEdsdmJpeHdj"
    "bTlxWldOMEtTQjdDaUFnYkdWMElITjBZWEowUFc1MWJHd3NaVzVrUFc1MWJHd3NjbVZ6ZFd4MFBXNTFiR3dzYzNSaGRHVTlJ"
    "blZ1YTI1dmQyNGlPd29nSUhSeWVYdHpkR0Z5ZEQxaGQyRnBkQ0JuWlhSRGJHOWpheWdwTzMxallYUmphSHRqYkdWaGJuVndS"
    "WEp5YjNJb0ltTnNiMk5yWDNWdVlYWmhhV3hoWW14bElpazdmUW9nSUdOdmJuTjBJR0ZqZEdsdmJqMTdiR0ZpWld3c2MzUmhj"
    "blJmWTJ4dlkyczZjM1JoY25Rc1pXNWtYMk5zYjJOck9tNTFiR3dzWVd4c2IzZGxaRjl0Y3pwaGJHeHZZMkYwYVc5dUxBb2dJ"
    "Q0FnY21WemRXeDBYM04wWVhSbE9pSjFibXR1YjNkdUlpeGxiR0Z3YzJWa1gyMXpPbTUxYkd3c2IzWmxjbDlpZFdSblpYUTZi"
    "blZzYkgwN0NpQWdjbVZqYjNKa0xtRmpkR2x2Ym5NdWNIVnphQ2hoWTNScGIyNHBPM0psZEdGcGJpZ3BPeUF2THlCVGRHRnlk"
    "Q0J5WlhSaGFXNWxaQ0JDUlVaUFVrVWdaVzUwWlhKcGJtY2dSSEpwZG1WeUlHTmhiR3d1Q2lBZ2RISjVJSHNLSUNBZ0lISmxj"
    "M1ZzZEQxaGQyRnBkQ0J2Y0dWeVlYUnBiMjRvS1RzS0lDQWdJSE4wWVhSbFBYSmxjM1ZzZEQ4dWFYTkZjbkp2Y2owOVBYUnlk"
    "V1UvSW5KbFpuVnpaV1FpT2lKeVpYUjFjbTVsWkNJN0NpQWdmU0JqWVhSamFDQjdjM1JoZEdVOUltVjRZMlZ3ZEdsdmJpSTdm"
    "UW9nSUhSeWVYdGxibVE5WVhkaGFYUWdaMlYwUTJ4dlkyc29LVHQ5WTJGMFkyaDdZMnhsWVc1MWNFVnljbTl5S0NKamJHOWph"
    "MTkxYm1GMllXbHNZV0pzWlNJcE8zMEtJQ0JoWTNScGIyNHVaVzVrWDJOc2IyTnJQV1Z1WkR0aFkzUnBiMjR1Y21WemRXeDBY"
    "M04wWVhSbFBYTjBZWFJsT3dvZ0lISmxkR0ZwYmlncE95QXZMeUJGYm1RdmNtVnpkV3gwSUhKbGRHRnBibVZrSUVKRlJrOVNS"
    "U0JpZFdSblpYUWdZMjl0Y0dGeWFYTnZiaTRLSUNCcFppaHpkR0Z5ZENZbVpXNWtLU0I3Q2lBZ0lDQmpiMjV6ZENCa1BXNXpL"
    "R1Z1WkNrdGJuTW9jM1JoY25RcE93b2dJQ0FnYVdZb1pENDlNRzRwSUhzS0lDQWdJQ0FnWVdOMGFXOXVMbVZzWVhCelpXUmZi"
    "WE05VG5WdFltVnlLR1F2TVRBd01EQXdNRzRwT3dvZ0lDQWdJQ0JoWTNScGIyNHViM1psY2w5aWRXUm5aWFE5WVdOMGFXOXVM"
    "bVZzWVhCelpXUmZiWE0rWVd4c2IyTmhkR2x2YmpzS0lDQWdJSDBnWld4elpTQmpiR1ZoYm5Wd1JYSnliM0lvSW1Oc2IyTnJY"
    "Mmx1ZG1Gc2FXUWlLVHNLSUNCOUNpQWdhV1lvWVdOMGFXOXVMbTkyWlhKZlluVmtaMlYwS1dOc1pXRnVkWEJGY25KdmNpZ2la"
    "SEpwZG1WeVgyOXdaWEpoZEdsdmJsOXZkbVZ5WDJKMVpHZGxkQ0lwT3dvZ0lHbG1LSE4wWVhSbElUMDlJbkpsZEhWeWJtVmtJ"
    "aWxqYkdWaGJuVndSWEp5YjNJb0ltUnlhWFpsY2w5dmNHVnlZWFJwYjI1ZmRXNWpiMjVtYVhKdFpXUWlLVHNLSUNCcFppaHdj"
    "bTlxWldOMEtYQnliMnBsWTNRb2NtVnpkV3gwTEhOMFlYUmxLVHNLSUNCeVpYUmhhVzRvS1RzS2ZRcGhjM2x1WXlCbWRXNWpk"
    "R2x2YmlCamJHVmhiblZ3S0NrZ2V3b2dJR2xtS0dOc1pXRnVkWEJUZEdGeWRHVmtLWEpsZEhWeWJqc0tJQ0JqYkdWaGJuVndV"
    "M1JoY25SbFpEMTBjblZsTzNKbFkyOXlaQzVqYkdWaGJuVndYM04wWVhKMFpXUTlkSEoxWlR0eVpYUmhhVzRvS1RzS0lDQjBj"
    "bmtnZXdvZ0lDQWdZMjl1YzNRZ2NqMWhkMkZwZENCc2IyTmhiQ2hUVkU5UUxIc0tJQ0FnSUNBZ2JHRmlPbTkzYmk1c1lXSXNa"
    "WFpsYm5SZmIzVjBPbTkzYmk1bGRtVnVkRjl2ZFhRc0NpQWdJQ0FnSUdacGNuTjBYMlpoYVd4MWNtVTZjbVZqYjNKa0xtWnBj"
    "bk4wWDJaaGFXeDFjbVVzQ2lBZ0lDQWdJR1pwY25OMFgyOWljMlZ5ZG1GMGFXOXVYM2RoYkd4ZmJYTTZjbVZqYjNKa0xtWnBj"
    "bk4wWDI5aWMyVnlkbUYwYVc5dVgzZGhiR3hmYlhNS0lDQWdJSDBwT3dvZ0lDQWdjbVZqYjNKa0xtWnBjbk4wWDJOc2IyTnJQ"
    "WEl1WTJ4dlkyczdjbVZqYjNKa0xuTjBiM0JmYzNSaGRHVTljaTV0WVhKclpYSmZjM1JoZEdVN2NtVjBZV2x1S0NrN0NpQWdJ"
    "Q0JwWmloeUxtVjJaVzUwWDNOMFlYUmxJVDA5SW5keWFYUjBaVzRpS1dOc1pXRnVkWEJGY25KdmNpZ2liMkp6WlhKMllYUnBi"
    "MjVmYldWMFlXUmhkR0ZmZDNKcGRHVmZkVzVqYjI1bWFYSnRaV1FpS1RzS0lDQWdJR2xtS0hJdWJXRnlhMlZ5WDNOMFlYUmxQ"
    "VDA5SW5OMGIzQmZkVzVqYjI1bWFYSnRaV1FpS1dOc1pXRnVkWEJGY25KdmNpZ2ljM1J2Y0Y5MWJtTnZibVpwY20xbFpDSXBP"
    "d29nSUNBZ2FXWW9jaTV0WVhKclpYSmZjM1JoZEdVOVBUMGliM2R1WlhKemFHbHdYM1Z1YTI1dmQyNGlLV05zWldGdWRYQkZj"
    "bkp2Y2lnaWIzZHVaWEp6YUdsd1gzVnVhMjV2ZDI0aUtUc0tJQ0I5SUdOaGRHTm9JSHRqYkdWaGJuVndSWEp5YjNJb0luTjBi"
    "M0JmYjNKZlkyeHZZMnRmY21WamIzSmtYM1Z1WVhaaGFXeGhZbXhsSWlrN2ZRb2dJQzh2SUU1dklHOTFkSE4wWVc1a2FXNW5J"
    "RVJ5YVhabGNpQmpZV3hzSUdWNGFYTjBjeUJvWlhKbE9pQmhiR3dnY0dGblpTQmpZV3hzY3lCaFltOTJaU0IzWlhKbElHRjNZ"
    "V2wwWldRdUNpQWdMeThnVDNKcFoybHVZV3dnYzNSdmNDQndjbTkwYjJOdmJDQmhiSEpsWVdSNUlHTmhkWE5sY3lCMGFHVWdZ"
    "Mjl1ZEhKdmJHeGxjaWR6SUc5M2JtVmtMV05vYVd4a0lHWnBibUZzYkhrdUNpQWdZWGRoYVhRZ2RHbHRaV1JFY21sMlpYSW9J"
    "bXRwYkd4ZllYQndJaXd6TURBd01Dd0tJQ0FnSUNncFBUNTBiMjlzY3k1dFkzQmZYMk4xWVY5a2NtbDJaWEpmWDJ0cGJHeGZZ"
    "WEJ3S0h0d2FXUTZiM2R1TG1KeWIzZHpaWEpmY0dsa2ZTa3BPd29nSUdGM1lXbDBJSFJwYldWa1JISnBkbVZ5S0NKbGJtUmZj"
    "MlZ6YzJsdmJpSXNNVFV3TURBc0NpQWdJQ0FvS1QwK2RHOXZiSE11YldOd1gxOWpkV0ZmWkhKcGRtVnlYMTlsYm1SZmMyVnpj"
    "Mmx2YmloN2MyVnpjMmx2YmpwdmQyNHVjMlZ6YzJsdmJuMHBMQ2h5TEhOMFlYUmxLVDArZXdvZ0lDQWdJQ0J5WldOdmNtUXVj"
    "MlZ6YzJsdmJsOWxibVJsWkQxemRHRjBaVDA5UFNKeVpYUjFjbTVsWkNJbUpnb2dJQ0FnSUNBZ0lISS9Mbk4wY25WamRIVnla"
    "V1JEYjI1MFpXNTBQeTVoWTNScGRtVTlQVDFtWVd4elpTWW1jaTV6ZEhKMVkzUjFjbVZrUTI5dWRHVnVkQzV6WlhOemFXOXVQ"
    "VDA5YjNkdUxuTmxjM05wYjI0N0NpQWdJQ0FnSUhKbGRHRnBiaWdwT3dvZ0lDQWdmU2s3Q2lBZ1lYZGhhWFFnZEdsdFpXUkVj"
    "bWwyWlhJb0lteHBjM1JmZDJsdVpHOTNjeUlzTlRBd01Dd0tJQ0FnSUNncFBUNTBiMjlzY3k1dFkzQmZYMk4xWVY5a2NtbDJa"
    "WEpmWDJ4cGMzUmZkMmx1Wkc5M2N5aDdjR2xrT205M2JpNWljbTkzYzJWeVgzQnBaSDBwTENoeUxITjBZWFJsS1QwK2V3b2dJ"
    "Q0FnSUNCamIyNXpkQ0IzYVc1a2IzZHpQWEkvTG5OMGNuVmpkSFZ5WldSRGIyNTBaVzUwUHk1M2FXNWtiM2R6T3dvZ0lDQWdJ"
    "Q0J5WldOdmNtUXVkMmx1Wkc5M1gyTnZkVzUwUFhOMFlYUmxQVDA5SW5KbGRIVnlibVZrSWlZbVFYSnlZWGt1YVhOQmNuSmhl"
    "U2gzYVc1a2IzZHpLVDkzYVc1a2IzZHpMbXhsYm1kMGFEcHVkV3hzT3dvZ0lDQWdJQ0J5WlhSaGFXNG9LVHNLSUNBZ0lIMHBP"
    "d29nSUd4bGRDQnlaV0ZrVTNSaGNuUTliblZzYkRzS0lDQjBjbmw3Y21WaFpGTjBZWEowUFdGM1lXbDBJR2RsZEVOc2IyTnJL"
    "Q2s3ZldOaGRHTm9lMk5zWldGdWRYQkZjbkp2Y2lnaVkyeHZZMnRmZFc1aGRtRnBiR0ZpYkdVaUtUdDlDaUFnWTI5dWMzUWdZ"
    "V04wYVc5dVBYdHNZV0psYkRvaWIzZHVaV1JmYW05cGJsOXlaV0ZrWW1GamF5SXNjM1JoY25SZlkyeHZZMnM2Y21WaFpGTjBZ"
    "WEowTEdWdVpGOWpiRzlqYXpwdWRXeHNMQW9nSUNBZ1lXeHNiM2RsWkY5dGN6cHVkV3hzTEdWc1lYQnpaV1JmYlhNNmJuVnNi"
    "Q3h2ZG1WeVgySjFaR2RsZERwdWRXeHNMSEpsYzNWc2RGOXpkR0YwWlRvaWRXNXJibTkzYmlKOU93b2dJSEpsWTI5eVpDNWhZ"
    "M1JwYjI1ekxuQjFjMmdvWVdOMGFXOXVLVHR5WlhSaGFXNG9LVHNLSUNCcFppaHlaV0ZrVTNSaGNuUW1KbkpsWTI5eVpDNW1h"
    "WEp6ZEY5amJHOWpheWtnZXdvZ0lDQWdZV04wYVc5dUxtRnNiRzkzWldSZmJYTTlUV0YwYUM1dFlYZ29NQ3cyTURBd01DMU9k"
    "VzFpWlhJb0tHNXpLSEpsWVdSVGRHRnlkQ2t0Ym5Nb2NtVmpiM0prTG1acGNuTjBYMk5zYjJOcktTa3ZNVEF3TURBd01HNHBL"
    "VHNLSUNBZ0lISmxkR0ZwYmlncE93b2dJSDBLSUNBdkx5QkRiMjUwYVc1MVpTQmxjM05sYm5ScFlXd2dhbTlwYmlCaFpuUmxj"
    "all3Y3lCcFppQnNZWFJsT3lCdVpYWmxjaUJqYkdGcGJTQjBhR0YwSUdSbFlXUnNhVzVsSUdWdVptOXlZMlZrTGdvZ0lDOHZJ"
    "RVJ2SUc1dmRDQndiMnhzSUdGdWIzUm9aWElnWlhobFl5QnpaWE56YVc5dUlHOXlJSE5sYm1RZ1lTQnRZVzUxWVd3Z2NISnZZ"
    "MlZ6Y3lCemFXZHVZV3d1Q2lBZ2QyaHBiR1VvSVdOdmJuUnliMnhzWlhKS2IybHVaV1FtSm1OdmJuUnliMnhzWlhKUWIyeHNR"
    "WFpoYVd4aFlteGxKaVpFWVhSbExtNXZkeWdwUEc5M2JpNXpkR0Z5ZEY5bGNHOWphRjl0Y3lzNU1EQXdNREFwSUhzS0lDQWdJ"
    "R0YzWVdsMElIQnZiR3hEYjI1MGNtOXNiR1Z5S0NrN0NpQWdJQ0JwWmloeVpXTnZjbVF1Wm1seWMzUmZZMnh2WTJzcElIc0tJ"
    "Q0FnSUNBZ2RISjVJSHNLSUNBZ0lDQWdJQ0JqYjI1emRDQmpQV0YzWVdsMElHZGxkRU5zYjJOcktDazdjbVZqYjNKa0xteGhj"
    "M1JmYW05cGJsOWpiRzlqYXoxak8zSmxkR0ZwYmlncE93b2dJQ0FnSUNBZ0lHbG1LRzV6S0dNcExXNXpLSEpsWTI5eVpDNW1h"
    "WEp6ZEY5amJHOWpheWsrTmpBd01EQXdNREF3TURCdUtRb2dJQ0FnSUNBZ0lDQWdZMnhsWVc1MWNFVnljbTl5S0NKamJHVmhi"
    "blZ3WDJKMVpHZGxkRjlsZUdObFpXUmxaQ0lwT3dvZ0lDQWdJQ0I5SUdOaGRHTm9lMk5zWldGdWRYQkZjbkp2Y2lnaVkyeHZZ"
    "MnRmZFc1aGRtRnBiR0ZpYkdVaUtUdDlDaUFnSUNCOUNpQWdmUW9nSUdsbUtDRmpiMjUwY205c2JHVnlTbTlwYm1Wa0tXTnNa"
    "V0Z1ZFhCRmNuSnZjaWdpWTJocGJHUmZjbVZoY0Y5cGJtTnZiWEJzWlhSbElpazdDaUFnZEhKNUlIc0tJQ0FnSUdOdmJuTjBJ"
    "SEk5WVhkaGFYUWdiRzlqWVd3b1VrVkJSRUpCUTBzc2V3b2dJQ0FnSUNCdmQyNWxaRjl3YVdSek9sdHZkMjR1WjNWaGNtUmZj"
    "R2xrTEc5M2JpNXpaWEoyWlhKZmNHbGtMRzkzYmk1aWNtOTNjMlZ5WDNCcFpDd0tJQ0FnSUNBZ0lDQXVMaTRvYjNkdUxtaGxi"
    "SEJsY2w5d2FXUTlQVDF1ZFd4c1AxdGRPbHR2ZDI0dWFHVnNjR1Z5WDNCcFpGMHBYU3dLSUNBZ0lDQWdiR0ZpT205M2JpNXNZ"
    "V0lzYjNWMFpYSmZiM1YwT205M2JpNXZkWFJsY2w5dmRYUXNhR1ZzY0dWeVgzQnBaRHB2ZDI0dWFHVnNjR1Z5WDNCcFpDeHpa"
    "WEoyWlhKZmNHbGtPbTkzYmk1elpYSjJaWEpmY0dsa0NpQWdJQ0I5S1RzS0lDQWdJR0ZqZEdsdmJpNWxibVJmWTJ4dlkyczlj"
    "aTVqYkc5amF6dGhZM1JwYjI0dWNtVnpkV3gwWDNOMFlYUmxQU0p5WlhSMWNtNWxaQ0k3Q2lBZ0lDQnlaV052Y21RdWIzZHVa"
    "V1JmWTJocGJHUmZaWGhwZEhNOWNpNXZkMjVsWkY5amFHbHNaRjlsZUdsMGN6c0tJQ0FnSUdScFlXZHViM04wYVdNb2NpNTFi"
    "bVY0Y0dWamRHVmtYMlpoYVd4MWNtVmZiMkp6WlhKMllYUnBiMjRzY2k1MWJtVjRjR1ZqZEdWa1gyOWljMlZ5ZG1GMGFXOXVY"
    "M0J5YjJwbFkzUnBiMjRwT3dvZ0lDQWdhV1lvVG5WdFltVnlMbWx6VTJGbVpVbHVkR1ZuWlhJb2NpNW9aV3h3WlhKZmNHbGtL"
    "U1ltY2k1b1pXeHdaWEpmY0dsa1BqQXBiM2R1TG1obGJIQmxjbDl3YVdROWNpNW9aV3h3WlhKZmNHbGtPd29nSUNBZ2NtVmpi"
    "M0prTG1acGJtRnNYMkZpYzJWdVkyVTllMjkzYm1Wa1gzQnBaSE02Y2k1dmQyNWxaRjl3YVdSekxIQnZjblJ6T25JdWNHOXlk"
    "SE1zQ2lBZ0lDQWdJR3hoWWpweUxteGhZbDloWW5ObGJuUXNkMmx1Wkc5M1gyTnZkVzUwT25KbFkyOXlaQzUzYVc1a2IzZGZZ"
    "MjkxYm5RL1AyNTFiR3dzQ2lBZ0lDQWdJSE5sYzNOcGIyNWZaVzVrWldRNmNtVmpiM0prTG5ObGMzTnBiMjVmWlc1a1pXUTlQ"
    "VDEwY25WbGZUc0tJQ0FnSUhKbFkyOXlaQzVtYVhKemRGOXZZbk5sY25aaGRHbHZibDkwYjE5bWFXNWhiRjkzWVd4c1gyMXpQ"
    "WEpsWTI5eVpDNW1hWEp6ZEY5dlluTmxjblpoZEdsdmJsOTNZV3hzWDIxelBUMDliblZzYkQ5dWRXeHNPZ29nSUNBZ0lDQkVZ"
    "WFJsTG01dmR5Z3BMWEpsWTI5eVpDNW1hWEp6ZEY5dlluTmxjblpoZEdsdmJsOTNZV3hzWDIxek93b2dJQ0FnY21WMFlXbHVL"
    "Q2s3SUM4dklFWjFiR3dnWm1sNFpXUWdjbVZoWkdKaFkyc3ZaWGhwZEhNdlkyeHZZMnNnUWtWR1QxSkZJR052YlhCaGNtbHpi"
    "MjV6TGdvZ0lDQWdhV1lvY21WaFpGTjBZWEowS1NCN0NpQWdJQ0FnSUdGamRHbHZiaTVsYkdGd2MyVmtYMjF6UFU1MWJXSmxj"
    "aWdvYm5Nb2NpNWpiRzlqYXlrdGJuTW9jbVZoWkZOMFlYSjBLU2t2TVRBd01EQXdNRzRwT3dvZ0lDQWdJQ0JoWTNScGIyNHVi"
    "M1psY2w5aWRXUm5aWFE5WVdOMGFXOXVMbUZzYkc5M1pXUmZiWE05UFQxdWRXeHNQMjUxYkd3NllXTjBhVzl1TG1Wc1lYQnpa"
    "V1JmYlhNK1lXTjBhVzl1TG1Gc2JHOTNaV1JmYlhNN0NpQWdJQ0I5Q2lBZ0lDQnBaaWh5WldOdmNtUXVabWx5YzNSZlkyeHZZ"
    "MnNwQ2lBZ0lDQWdJSEpsWTI5eVpDNXNZWFJqYUY5MGIxOWhZbk5sYm1ObFgyMXpQVTUxYldKbGNpZ29ibk1vY2k1amJHOWph"
    "eWt0Ym5Nb2NtVmpiM0prTG1acGNuTjBYMk5zYjJOcktTa3ZNVEF3TURBd01HNHBPd29nSUNBZ2FXWW9ZV04wYVc5dUxtOTJa"
    "WEpmWW5Wa1oyVjBLV05zWldGdWRYQkZjbkp2Y2lnaVkyeGxZVzUxY0Y5aWRXUm5aWFJmWlhoalpXVmtaV1FpS1RzS0lDQWdJ"
    "R052Ym5OMElHRTljbVZqYjNKa0xtWnBibUZzWDJGaWMyVnVZMlU3Q2lBZ0lDQnBaaWdoWTI5dWRISnZiR3hsY2twdmFXNWxa"
    "SHg4SVVGeWNtRjVMbWx6UVhKeVlYa29jbVZqYjNKa0xtOTNibVZrWDJOb2FXeGtYMlY0YVhSektYeDhDaUFnSUNBZ0lDQnla"
    "V052Y21RdWIzZHVaV1JmWTJocGJHUmZaWGhwZEhNdWJHVnVaM1JvSVQwOUtHOTNiaTVvWld4d1pYSmZjR2xrUFQwOWJuVnNi"
    "RDgyT2pjcEtRb2dJQ0FnSUNCamJHVmhiblZ3UlhKeWIzSW9JbU5vYVd4a1gzSmxZWEJmYVc1amIyMXdiR1YwWlNJcE93b2dJ"
    "Q0FnYVdZb0lVOWlhbVZqZEM1MllXeDFaWE1vWVM1dmQyNWxaRjl3YVdSektTNWxkbVZ5ZVNoMlBUNTJQVDA5ZEhKMVpTbDhm"
    "QW9nSUNBZ0lDQWdJVTlpYW1WamRDNTJZV3gxWlhNb1lTNXdiM0owY3lrdVpYWmxjbmtvZGowK2RqMDlQWFJ5ZFdVcGZIeGhM"
    "bXhoWWlFOVBYUnlkV1Y4ZkFvZ0lDQWdJQ0FnWVM1M2FXNWtiM2RmWTI5MWJuUWhQVDB3Zkh4aExuTmxjM05wYjI1ZlpXNWta"
    "V1FoUFQxMGNuVmxLUW9nSUNBZ0lDQmpiR1ZoYm5Wd1JYSnliM0lvSW05M2JtVmtYM0psYzI5MWNtTmxYMkZpYzJWdVkyVmZk"
    "VzV3Y205MlpXNGlLVHNLSUNCOUlHTmhkR05vSUh0aFkzUnBiMjR1Y21WemRXeDBYM04wWVhSbFBTSmxlR05sY0hScGIyNGlP"
    "Mk5zWldGdWRYQkZjbkp2Y2lnaWIzZHVaV1JmY21WaFpHSmhZMnRmZFc1aGRtRnBiR0ZpYkdVaUtUdDlDaUFnWTJ4bFlXNTFj"
    "RVZ5Y205eUtDSm1hWEp6ZEY5bGRtVnVkRjkxYm5CeWIzWmxiaUlwT3dvZ0lISmxZMjl5WkM1M2FHOXNaVjlqYkdWaGJuVndY"
    "M2RwZEdocGJqWXdYM0J5YjNabGJqMW1ZV3h6WlR0eVpYUmhhVzRvS1RzS0lDQXZMeUJGZUdOc2RYTnBkbVVnYm1WM0lHWnBi"
    "R1VnYjI1c2VUc2daWGhwYzNScGJtY2diM1YwWlhJdmFHVnNjR1Z5TDNCeWIzWnBaR1Z5SUdWMmFXUmxibU5sSUhWdWRHOTFZ"
    "MmhsWkM0S0lDQjBjbmtnZXdvZ0lDQWdZMjl1YzNRZ1kyMWtQU0p3ZVhSb2IyNHpJQzFqSUNJcmMzRW9VRVZTVTBsVFZDa3JJ"
    "aUFpSzNOeEtHOTNiaTVqYkdWaGJuVndYMjkxZENrcklpQWlLM054S0VwVFQwNHVjM1J5YVc1bmFXWjVLSEpsWTI5eVpDa3BP"
    "d29nSUNBZ1kyOXVjM1FnY2oxaGQyRnBkQ0IwYjI5c2N5NWxlR1ZqWDJOdmJXMWhibVFvZTJOdFpDeDNiM0pyWkdseU9tOTNi"
    "aTUzYjNKcmMzQmhZMlVzQ2lBZ0lDQWdJSGxwWld4a1gzUnBiV1ZmYlhNNk1UQXdNREFzYldGNFgyOTFkSEIxZEY5MGIydGxi"
    "bk02TlRBd2ZTazdDaUFnSUNCeVpXTnZjbVF1Ykc5allXeGZkRzl2YkY5eVpXTmxhWEIwY3k1d2RYTm9LSHRzWVdKbGJEb2lj"
    "R1Z5YzJsemRDSXNDaUFnSUNBZ0lHVjRhWFE2ZEhsd1pXOW1JSEl1WlhocGRGOWpiMlJsUFQwOUltNTFiV0psY2lJL2NpNWxl"
    "R2wwWDJOdlpHVTZiblZzYkN3S0lDQWdJQ0FnYzJWemMybHZibDlwWkRwMGVYQmxiMllnY2k1elpYTnphVzl1WDJsa1BUMDlJ"
    "bTUxYldKbGNpSS9jaTV6WlhOemFXOXVYMmxrT201MWJHeDlLVHR5WlhSaGFXNG9LVHNLSUNBZ0lHbG1LSEl1YzJWemMybHZi"
    "bDlwWkNFOVBYVnVaR1ZtYVc1bFpDbGpiR1ZoYm5Wd1JYSnliM0lvSW05M2JtVmtYMnh2WTJGc1gyTnZiVzFoYm1SZmRXNXFi"
    "Mmx1WldRaUtUc0tJQ0FnSUdsbUtISXVjMlZ6YzJsdmJsOXBaQ0U5UFhWdVpHVm1hVzVsWkh4OGNpNWxlR2wwWDJOdlpHVWhQ"
    "VDB3S1FvZ0lDQWdJQ0JqYkdWaGJuVndSWEp5YjNJb0ltTnNaV0Z1ZFhCZmJXVjBZV1JoZEdGZmQzSnBkR1ZmZFc1amIyNW1h"
    "WEp0WldRaUtUc0tJQ0I5SUdOaGRHTm9JSHRqYkdWaGJuVndSWEp5YjNJb0ltTnNaV0Z1ZFhCZmJXVjBZV1JoZEdGZmQzSnBk"
    "R1ZmZFc1amIyNW1hWEp0WldRaUtUdDlDaUFnWTI5dWRISnZiR3hsY2tKMVptWmxjajBpSWp0dmQyNHVZMnh2YzJWa1BYUnlk"
    "V1U3YzNSdmNtVW9JbVF3TVY5cGJXMWxaR2xoZEdWZmIzZHVaV1JmYUdGdVpHeGxjeUlzYjNkdUtUc0tmUXBoYzNsdVl5Qm1k"
    "VzVqZEdsdmJpQmphR1ZqYTJWa0tHeGhZbVZzTEc5d1pYSmhkR2x2Yml4d2NtVmthV05oZEdVcElIc0tJQ0JwWmloeVpXTnZj"
    "bVF1Wm1seWMzUmZabUZwYkhWeVpTRTlQVzUxYkd3cGNtVjBkWEp1SUc1MWJHdzdDaUFnYVdZb1JHRjBaUzV1YjNjb0tUNDli"
    "M2R1TG5OMFlYSjBYMlZ3YjJOb1gyMXpLemcwTURBd01Da2dld29nSUNBZ2JHRjBZMmdvSW1KeWIzZHpaWEpmWVdOMGFYWmxY"
    "MlJsWVdSc2FXNWxJaXhFWVhSbExtNXZkeWdwS1R0aGQyRnBkQ0JqYkdWaGJuVndLQ2s3Y21WMGRYSnVJRzUxYkd3N0NpQWdm"
    "UW9nSUd4bGRDQnlaWE4xYkhRc2NtVmpaV2wyWldSWFlXeHNPd29nSUhSeWVTQjdDaUFnSUNCeVpYTjFiSFE5WVhkaGFYUWdi"
    "M0JsY21GMGFXOXVLQ2s3Y21WalpXbDJaV1JYWVd4c1BVUmhkR1V1Ym05M0tDazdDaUFnSUNCeVpXTnZjbVF1YjJKelpYSjJZ"
    "WFJwYjI1ZmNtVmpaV2x3ZEhNdWNIVnphQ2g3YTJsdVpEcHNZV0psYkN4eVpXTmxhWFpsWkY5M1lXeHNYMjF6T25KbFkyVnBk"
    "bVZrVjJGc2JIMHBPM0psZEdGcGJpZ3BPd29nSUNBZ2FXWW9jbVZ6ZFd4MFB5NXBjMFZ5Y205eVBUMDlkSEoxWlNsc1lYUmph"
    "Q2dpWW5KdmQzTmxjbDkwYjI5c1gzSmxablZ6WldRaUxISmxZMlZwZG1Wa1YyRnNiQ2s3Q2lBZ0lDQmxiSE5sSUdsbUtDRndj"
    "bVZrYVdOaGRHVW9jbVZ6ZFd4MEtTbHNZWFJqYUNnaVluSnZkM05sY2w5a1pXTnBjMmx2Ymw5MWJtTnZibVpwY20xbFpDSXNj"
    "bVZqWldsMlpXUlhZV3hzS1RzS0lDQjlJR05oZEdOb0lIdHNZWFJqYUNnaVluSnZkM05sY2w5MGIyOXNYMjl5WDJSbFkybHph"
    "Vzl1WDJWNFkyVndkR2x2YmlJc1JHRjBaUzV1YjNjb0tTazdmUW9nSUdsbUtISmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxJ"
    "VDA5Ym5Wc2JDbGhkMkZwZENCamJHVmhiblZ3S0NrN0NpQWdjbVYwZFhKdUlISmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxQ"
    "VDA5Ym5Wc2JEOXlaWE4xYkhRNmJuVnNiRHNLZlFwamIyNXpkQ0J6Ym1Gd2MyaHZkRUZ5WjNNOWUzTmxjM05wYjI0NmIzZHVM"
    "bk5sYzNOcGIyNHNkR0Z5WjJWMFgybGtPbTkzYmk1MFlYSm5aWFJmYVdRc2RHRmlYMmxrT205M2JpNTBZV0pmYVdRc0NpQWdj"
    "MjVoY0hOb2IzUmZabTl5YldGME9pSnpaVzFoYm5ScFkxOTJNaUlzYVc1amJIVmtaVjl6WTNKbFpXNXphRzkwT21aaGJITmxm"
    "VHNLWm5WdVkzUnBiMjRnY0dGblpTaHlaWE4xYkhRc2EybHVaQ2tnZXdvZ0lHTnZibk4wSUhNOWNtVnpkV3gwUHk1emRISjFZ"
    "M1IxY21Wa1EyOXVkR1Z1ZEN4dWIyUmxjejFCY25KaGVTNXBjMEZ5Y21GNUtITS9MbU52Ym5SbGJuUmZjbVZtY3lrL2N5NWpi"
    "MjUwWlc1MFgzSmxabk02VzEwN0NpQWdZMjl1YzNRZ1pteGhaM005ZXdvZ0lDQWdjM1JoZEhWelgyOXJPbkpsYzNWc2REOHVh"
    "WE5GY25KdmNpRTlQWFJ5ZFdVbUpuTS9Mbk4wWVhSMWN6MDlQU0p2YXlJc0NpQWdJQ0JqYjIxd2JHVjBaVHB6UHk1emJtRndj"
    "Mmh2ZEQ4dVkyOXRjR3hsZEdVOVBUMTBjblZsTEFvZ0lDQWdhR1ZoWkdsdVoxOXRZWFJqYURwdWIyUmxjeTV6YjIxbEtHNDlQ"
    "bTR1Y205c1pUMDlQU0pvWldGa2FXNW5JaVltYmk1dVlXMWxQVDA5Q2lBZ0lDQWdJQ2hyYVc1a1BUMDlJbkJ5YjNSbFkzUmxa"
    "RjloWm5SbGNpSS9JbEJ5YjNSbFkzUmxaQ0JoY0hCc2FXTmhkR2x2YmlCaFkyTmxjM01pT2lKTWIyTmhiQ0JrWlcxdklpa3BM"
    "QW9nSUNBZ1pYSnliM0pmYldGMFkyZzZibTlrWlhNdWMyOXRaU2h1UFQ1dUxtNWhiV1U5UFQwaVRHOWpZV3dnWkdWdGJ5Qmpi"
    "M1ZzWkNCdWIzUWdZMjl0Y0d4bGRHVWdkR2hwY3lCeVpYRjFaWE4wTGlJcExBb2dJQ0FnY21WeGRXbHlaV1JmZEdWNGREcHVi"
    "MlJsY3k1emIyMWxLRzQ5UG00dWJtRnRaVDA5UFNocmFXNWtQVDA5SW5CeWIzUmxZM1JsWkY5aVpXWnZjbVVpUHlKVGFXZHVJ"
    "R2x1SUhKbGNYVnBjbVZrTGlJNkNpQWdJQ0FnSUd0cGJtUTlQVDBpY0hKdmRHVmpkR1ZrWDJGbWRHVnlJajhpVTJsbmJtVmtJ"
    "R2x1TGlCUWNtOTBaV04wWldRZ1lYQndiR2xqWVhScGIyNGdZV05qWlhOeklHbHpJR0YyWVdsc1lXSnNaUzRpT2dvZ0lDQWdJ"
    "Q0FpVlhObElGTnBaMjRnYVc0Z2RHOGdiM0JsYmlCMGFHbHpJR3h2WTJGc0lHRndjR3hwWTJGMGFXOXVMaUlwS1N3S0lDQWdJ"
    "R2x1ZEdWeVlXTjBhWFpsWDNKbFpsOXdjbVZ6Wlc1ME9rRnljbUY1TG1selFYSnlZWGtvY3o4dWNtVm1jeWttSmdvZ0lDQWdJ"
    "Q0J6TG5KbFpuTXVjMjl0WlNodVBUNUJjbkpoZVM1cGMwRnljbUY1S0c0dVlXTjBhVzl1Y3lrbUptNHVZV04wYVc5dWN5NXBi"
    "bU5zZFdSbGN5Z2lZMnhwWTJzaUtTa0tJQ0I5T3dvZ0lISmxZMjl5WkM1c1lYTjBYM0JoWjJWZlpteGhaM005ZTJ0cGJtUXNM"
    "aTR1Wm14aFozTjlPM0psZEdGcGJpZ3BPd29nSUM4dklFOXViSGtnY0hKdmRtbGtaWElnY21WbWN5QmhibVFnWm1sNFpXUWdj"
    "SFZpYkdsakxXeGhZbVZzSUcxaGRHTm9aWE1nYzNWeWRtbDJaU0IwYUdseklISmhkeUJ6Ym1Gd2MyaHZkQzRLSUNCdmQyNHVa"
    "bkpsYzJoZmNtVm1jejFCY25KaGVTNXBjMEZ5Y21GNUtITS9MbkpsWm5NcFAzTXVjbVZtY3k1bWFXeDBaWElvYmowK2RIbHda"
    "VzltSUc0dWNtVm1QVDA5SW5OMGNtbHVaeUltSmdvZ0lDQWdMMTV3V3pBdE9WMHJPbHN3TFRsZEt5UXZMblJsYzNRb2JpNXla"
    "V1lwS1M1dFlYQW9iajArYmk1eVpXWXBPbHRkT3dvZ0lITjBiM0psS0NKa01ERmZhVzF0WldScFlYUmxYMjkzYm1Wa1gyaGhi"
    "bVJzWlhNaUxHOTNiaWs3Q2lBZ2FXWW9abXhoWjNNdVpYSnliM0pmYldGMFkyZ3BjbVYwZFhKdUlHWmhiSE5sT3dvZ0lHbG1L"
    "Q0ZtYkdGbmN5NXpkR0YwZFhOZmIydDhmQ0ZtYkdGbmN5NWpiMjF3YkdWMFpTbHlaWFIxY200Z1ptRnNjMlU3Q2lBZ2FXWW9h"
    "Mmx1WkQwOVBTSm5aVzVsY21salgyTm9aV05yWldRaUtTQjdDaUFnSUNCcFppaHViMlJsY3k1emIyMWxLRzQ5UG00dWNtOXNa"
    "VDA5UFNKb1pXRmthVzVuSWlZbWJpNXVZVzFsUFQwOUlsQnliM1JsWTNSbFpDQmhjSEJzYVdOaGRHbHZiaUJoWTJObGMzTWlL"
    "U1ltQ2lBZ0lDQWdJQ0J1YjJSbGN5NXpiMjFsS0c0OVBtNHVibUZ0WlQwOVBTSlRhV2R1WldRZ2FXNHVJRkJ5YjNSbFkzUmxa"
    "Q0JoY0hCc2FXTmhkR2x2YmlCaFkyTmxjM01nYVhNZ1lYWmhhV3hoWW14bExpSXBLUW9nSUNBZ0lDQnlaV052Y21RdWNISnZk"
    "R1ZqZEdWa1gyRm1kR1Z5WDNCaFoyVTlkSEoxWlRzS0lDQWdJSEpsZEhWeWJpQjBjblZsT3dvZ0lIMEtJQ0JqYjI1emRDQnZh"
    "ejFtYkdGbmN5NW9aV0ZrYVc1blgyMWhkR05vSmlabWJHRm5jeTV5WlhGMWFYSmxaRjkwWlhoMEppWUtJQ0FnSUNocmFXNWtJ"
    "VDA5SW1Gd2NHeHBZMkYwYVc5dUlueDhabXhoWjNNdWFXNTBaWEpoWTNScGRtVmZjbVZtWDNCeVpYTmxiblFwT3dvZ0lHbG1L"
    "RzlySmlacmFXNWtQVDA5SW5CeWIzUmxZM1JsWkY5aFpuUmxjaUlwY21WamIzSmtMbkJ5YjNSbFkzUmxaRjloWm5SbGNsOXdZ"
    "V2RsUFhSeWRXVTdDaUFnY21WMGRYSnVJRzlyT3dwOUNtRnplVzVqSUdaMWJtTjBhVzl1SUc1aGRtbG5ZWFJsUVc1a1UyNWhj"
    "SE5vYjNRb2RYSnNMR3RwYm1RcElIc0tJQ0JwWmloaGQyRnBkQ0JqYUdWamEyVmtLQ0ppY205M2MyVnlYMjVoZG1sbllYUnBi"
    "MjRpTEFvZ0lDQWdJQ0FvS1QwK2RHOXZiSE11YldOd1gxOWpkV0ZmWkhKcGRtVnlYMTlpY205M2MyVnlYMjVoZG1sbllYUmxL"
    "SHR6WlhOemFXOXVPbTkzYmk1elpYTnphVzl1TEFvZ0lDQWdJQ0FnSUhSaGNtZGxkRjlwWkRwdmQyNHVkR0Z5WjJWMFgybGtM"
    "SFJoWWw5cFpEcHZkMjR1ZEdGaVgybGtMSFZ5YkgwcExBb2dJQ0FnSUNCeVBUNXlQeTVwYzBWeWNtOXlJVDA5ZEhKMVpTa3BJ"
    "SHNLSUNBZ0lHRjNZV2wwSUdOb1pXTnJaV1FvSW1KeWIzZHpaWEpmYzI1aGNITm9iM1FpTEFvZ0lDQWdJQ0FvS1QwK2RHOXZi"
    "SE11YldOd1gxOWpkV0ZmWkhKcGRtVnlYMTluWlhSZlluSnZkM05sY2w5emRHRjBaU2h6Ym1Gd2MyaHZkRUZ5WjNNcExISTlQ"
    "bkJoWjJVb2NpeHJhVzVrS1NrN0NpQWdmUXA5Q21GemVXNWpJR1oxYm1OMGFXOXVJR1Z1ZEhKNUtDa2dld29nSUM4dklGUm9a"
    "U0JsZUdGamRDQkVjbWwyWlhJdGIzZHVaV1FnWW14aGJtc2dhR0Z1Wkd4bGN5QmhiSEpsWVdSNUlHVjRhWE4wSUhkb2FXeGxJ"
    "SFJvWlRFNE1ITWdaMkYwWlNCM1lXbDBjeTRLSUNBdkx5QlVhR1Z5WlNCcGN5QnVieUJ0YjJSbGJDQjVhV1ZzWkN3Z2RHOXZi"
    "QzFrWlhOamNtbHdkR2x2YmlCc2IyRmtJRzl5SUhKbFltbHVaQ0JoWm5SbGNpQjBhR2x6SUcxaGNtdGxjaTRLSUNCamIyNXpk"
    "Q0J0WVhKclpYSkRiMjF0WVc1a1BTSndlWFJvYjI0eklDMWpJQ0lyYzNFb1RVRlNTMFZTS1NzaUlDSXJjM0VvYjNkdUxteGhZ"
    "aWtySWlBaUt3b2dJQ0FnYzNFb2IzZHVMbWQxWVhKa1gzQnBaQ2tySWlBaUszTnhLRzkzYmk1elpYSjJaWEpmY0dsa0tUc0tJ"
    "Q0JoZDJGcGRDQmphR1ZqYTJWa0tDSndjbVZ3WVhKbFpGOXRZWEpyWlhJaUxBb2dJQ0FnS0NrOVBuUnZiMnh6TG1WNFpXTmZZ"
    "Mjl0YldGdVpDaDdZMjFrT20xaGNtdGxja052YlcxaGJtUXNkMjl5YTJScGNqcHZkMjR1ZDI5eWEzTndZV05sTEFvZ0lDQWdJ"
    "Q0I1YVdWc1pGOTBhVzFsWDIxek9qRXdNREF3TEcxaGVGOXZkWFJ3ZFhSZmRHOXJaVzV6T2pVd01IMHBMSEk5UG5zS0lDQWdJ"
    "Q0FnY21WamIzSmtMbXh2WTJGc1gzUnZiMnhmY21WalpXbHdkSE11Y0hWemFDaDdiR0ZpWld3NkltMWhjbXRsY2lJc0NpQWdJ"
    "Q0FnSUNBZ1pYaHBkRHAwZVhCbGIyWWdjaTVsZUdsMFgyTnZaR1U5UFQwaWJuVnRZbVZ5SWo5eUxtVjRhWFJmWTI5a1pUcHVk"
    "V3hzTEFvZ0lDQWdJQ0FnSUhObGMzTnBiMjVmYVdRNmRIbHdaVzltSUhJdWMyVnpjMmx2Ymw5cFpEMDlQU0p1ZFcxaVpYSWlQ"
    "M0l1YzJWemMybHZibDlwWkRwdWRXeHNmU2s3Y21WMFlXbHVLQ2s3Q2lBZ0lDQWdJR2xtS0hJdWMyVnpjMmx2Ymw5cFpDRTlQ"
    "WFZ1WkdWbWFXNWxaQ2xqYkdWaGJuVndSWEp5YjNJb0ltOTNibVZrWDJ4dlkyRnNYMk52YlcxaGJtUmZkVzVxYjJsdVpXUWlL"
    "VHNLSUNBZ0lDQWdjbVYwZFhKdUlISXVaWGhwZEY5amIyUmxQVDA5TUNZbWNpNXpaWE56YVc5dVgybGtQVDA5ZFc1a1pXWnBi"
    "bVZrSmlZS0lDQWdJQ0FnSUNCS1UwOU9MbkJoY25ObEtISXViM1YwY0hWMEtTNXdjbVZ3WVhKbFpGOXRZWEpyWlhKZmQzSnBk"
    "SFJsYmowOVBYUnlkV1U3Q2lBZ0lDQjlLVHNLSUNCamIyNXpkQ0J5WldGa2VVUmxZV1JzYVc1bFBVUmhkR1V1Ym05M0tDa3JN"
    "ekF3TURBN0NpQWdkMmhwYkdVb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVTlQVDF1ZFd4c0ppWnZkMjR1Wm1sNGRIVnla"
    "Vjl5WldGa2VTRTlQWFJ5ZFdVbUprUmhkR1V1Ym05M0tDazhjbVZoWkhsRVpXRmtiR2x1WlNrS0lDQWdJR0YzWVdsMElIQnZi"
    "R3hEYjI1MGNtOXNiR1Z5S0NrN0NpQWdhV1lvY21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21VOVBUMXVkV3hzSmladmQyNHVa"
    "bWw0ZEhWeVpWOXlaV0ZrZVNFOVBYUnlkV1VwQ2lBZ0lDQnNZWFJqYUNnaVptbDRkSFZ5WlY5eVpXRmtlVjkxYm1OdmJtWnBj"
    "bTFsWkNJc1JHRjBaUzV1YjNjb0tTazdDaUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVWhQVDF1ZFd4c0tYdGhk"
    "MkZwZENCamJHVmhiblZ3S0NrN2NtVjBkWEp1TzMwS0lDQmhkMkZwZENCd2IyeHNRMjl1ZEhKdmJHeGxjaWdwT3lBdkx5QlBU"
    "a1VnYVcxdFpXUnBZWFJsSUhCeVpTMXVZWFpwWjJGMGFXOXVJSEJ2Ykd3c0lHNXZJRkpRSUVoVVZGQWdjSEp2WW1VdUNpQWdh"
    "V1lvY21WamIzSmtMbVpwY25OMFgyWmhhV3gxY21VaFBUMXVkV3hzS1h0aGQyRnBkQ0JqYkdWaGJuVndLQ2s3Y21WMGRYSnVP"
    "MzBLSUNCaGQyRnBkQ0J1WVhacFoyRjBaVUZ1WkZOdVlYQnphRzkwS0NKb2RIUndPaTh2Ykc5allXeG9iM04wT2pNd01EQXZj"
    "SEp2ZEdWamRHVmtJaXdpY0hKdmRHVmpkR1ZrWDJKbFptOXlaU0lwT3dvZ0lHbG1LSEpsWTI5eVpDNW1hWEp6ZEY5bVlXbHNk"
    "WEpsUFQwOWJuVnNiQ2xoZDJGcGRDQndiMnhzUTI5dWRISnZiR3hsY2lncE93b2dJR2xtS0hKbFkyOXlaQzVtYVhKemRGOW1Z"
    "V2xzZFhKbElUMDliblZzYkNsN1lYZGhhWFFnWTJ4bFlXNTFjQ2dwTzNKbGRIVnlianQ5Q2lBZ1lYZGhhWFFnYm1GMmFXZGhk"
    "R1ZCYm1SVGJtRndjMmh2ZENnaWFIUjBjRG92TDJ4dlkyRnNhRzl6ZERvek1EQXdMeUlzSW1Gd2NHeHBZMkYwYVc5dUlpazdD"
    "aUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVTlQVDF1ZFd4c0tXRjNZV2wwSUhCdmJHeERiMjUwY205c2JHVnlL"
    "Q2s3SUM4dklFOU9SU0J3YjNOMExXVnVkSEo1SUhCdmJHd3VDaUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVTlQ"
    "VDF1ZFd4c0tTQjdDaUFnSUNCdmQyNHVjR2hoYzJVOUltSnliM2R6WlhKZlpHVmphWE5wYjI0aU93b2dJQ0FnYzNSdmNtVW9J"
    "bVF3TVY5cGJXMWxaR2xoZEdWZmIzZHVaV1JmYUdGdVpHeGxjeUlzYjNkdUtUc0tJQ0I5Q2lBZ2FXWW9jbVZqYjNKa0xtWnBj"
    "bk4wWDJaaGFXeDFjbVVoUFQxdWRXeHNLV0YzWVdsMElHTnNaV0Z1ZFhBb0tUc0tmUXBoYzNsdVl5Qm1kVzVqZEdsdmJpQmpi"
    "MjUwYVc1MVlYUnBiMjRvS1NCN0NpQWdZMjl1YzNRZ2MzQmxZejF2ZDI0dWJtVjRkRjlrWldOcGMybHZianNLSUNCamIyNXpk"
    "Q0JoYkd4dmQyVmtTMmx1WkhNOVd5SmpiR2xqYXlJc0luVnpaWEp1WVcxbElpd2ljR0Z6YzNkdmNtUWlMQ0p6Ym1Gd2MyaHZk"
    "Q0lzSW5CeWIzUmxZM1JsWkY5aFpuUmxjaUpkT3dvZ0lHbG1LQ0Z6Y0dWamZId2hZV3hzYjNkbFpFdHBibVJ6TG1sdVkyeDFa"
    "R1Z6S0hOd1pXTXVhMmx1WkNrcElIc0tJQ0FnSUd4aGRHTm9LQ0ppY205M2MyVnlYMlJsWTJsemFXOXVYM1Z1WTI5dVptbHli"
    "V1ZrSWl4RVlYUmxMbTV2ZHlncEtUdGhkMkZwZENCamJHVmhiblZ3S0NrN2NtVjBkWEp1T3dvZ0lIMEtJQ0JoZDJGcGRDQndi"
    "MnhzUTI5dWRISnZiR3hsY2lncE93b2dJR2xtS0hKbFkyOXlaQzVtYVhKemRGOW1ZV2xzZFhKbElUMDliblZzYkNsN1lYZGhh"
    "WFFnWTJ4bFlXNTFjQ2dwTzNKbGRIVnlianQ5Q2lBZ2FXWW9jM0JsWXk1cmFXNWtQVDA5SW5CeWIzUmxZM1JsWkY5aFpuUmxj"
    "aUlwSUhzS0lDQWdJQzh2SUZKbFlXUWdkR2hsSUdaeVpYTm9JR05oYkd4aVlXTnJMV1p2Ykd4dmQybHVaeUJ3Y205MFpXTjBa"
    "V1FnY0dGblpUc2daRzhnYm05MElITmxibVFnWVc1dmRHaGxjaUJTVUNCeVpYRjFaWE4wTGdvZ0lDQWdZWGRoYVhRZ1kyaGxZ"
    "MnRsWkNnaVluSnZkM05sY2w5emJtRndjMmh2ZENJc0NpQWdJQ0FnSUNncFBUNTBiMjlzY3k1dFkzQmZYMk4xWVY5a2NtbDJa"
    "WEpmWDJkbGRGOWljbTkzYzJWeVgzTjBZWFJsS0hOdVlYQnphRzkwUVhKbmN5a3NjajArY0dGblpTaHlMQ0p3Y205MFpXTjBa"
    "V1JmWVdaMFpYSWlLU2s3Q2lBZ2ZTQmxiSE5sSUdsbUtITndaV011YTJsdVpEMDlQU0p6Ym1Gd2MyaHZkQ0lwSUhzS0lDQWdJ"
    "R0YzWVdsMElHTm9aV05yWldRb0ltSnliM2R6WlhKZmMyNWhjSE5vYjNRaUxBb2dJQ0FnSUNBb0tUMCtkRzl2YkhNdWJXTndY"
    "MTlqZFdGZlpISnBkbVZ5WDE5blpYUmZZbkp2ZDNObGNsOXpkR0YwWlNoemJtRndjMmh2ZEVGeVozTXBMQW9nSUNBZ0lDQnlQ"
    "VDV3WVdkbEtISXNJbWRsYm1WeWFXTmZZMmhsWTJ0bFpDSXBKaVp3ZFdKc2FXTlFjbVZrYVdOaGRHVW9jaXh6Y0dWaktTazdD"
    "aUFnZlNCbGJITmxJSHNLSUNBZ0lHbG1LSFI1Y0dWdlppQnpjR1ZqTG5KbFppRTlQU0p6ZEhKcGJtY2lmSHdoUVhKeVlYa3Vh"
    "WE5CY25KaGVTaHZkMjR1Wm5KbGMyaGZjbVZtY3lsOGZBb2dJQ0FnSUNBZ0lXOTNiaTVtY21WemFGOXlaV1p6TG1sdVkyeDFa"
    "R1Z6S0hOd1pXTXVjbVZtS1NrZ2V3b2dJQ0FnSUNCc1lYUmphQ2dpWW5KdmQzTmxjbDlrWldOcGMybHZibDkxYm1OdmJtWnBj"
    "bTFsWkNJc1JHRjBaUzV1YjNjb0tTazdZWGRoYVhRZ1kyeGxZVzUxY0NncE8zSmxkSFZ5YmpzS0lDQWdJSDBLSUNBZ0lHOTNi"
    "aTVtY21WemFGOXlaV1p6UFZ0ZE8zTjBiM0psS0NKa01ERmZhVzF0WldScFlYUmxYMjkzYm1Wa1gyaGhibVJzWlhNaUxHOTNi"
    "aWs3SUM4dklGTnBibWRzWlMxMWMyVWdjMjVoY0hOb2IzUWdjbVZtTGdvZ0lDQWdZMjl1YzNRZ1lYSm5jejE3YzJWemMybHZi"
    "anB2ZDI0dWMyVnpjMmx2Yml4MFlYSm5aWFJmYVdRNmIzZHVMblJoY21kbGRGOXBaQ3gwWVdKZmFXUTZiM2R1TG5SaFlsOXBa"
    "Q3h5WldZNmMzQmxZeTV5WldaOU93b2dJQ0FnWTI5dWMzUWdiM0JsY21GMGFXOXVQWE53WldNdWEybHVaRDA5UFNKamJHbGph"
    "eUkvQ2lBZ0lDQWdJQ2dwUFQ1MGIyOXNjeTV0WTNCZlgyTjFZVjlrY21sMlpYSmZYMkp5YjNkelpYSmZZMnhwWTJzb2V5NHVM"
    "bUZ5WjNNc2FXNXdkWFJmY205MWRHVTZJbVJ2YlY5bGRtVnVkQ0o5S1RvS0lDQWdJQ0FnS0NrOVBuUnZiMnh6TG0xamNGOWZZ"
    "M1ZoWDJSeWFYWmxjbDlmWW5KdmQzTmxjbDkwZVhCbEtIc3VMaTVoY21kekxISmxjR3hoWTJVNmRISjFaU3dLSUNBZ0lDQWdJ"
    "Q0IwWlhoME9uTndaV011YTJsdVpEMDlQU0oxYzJWeWJtRnRaU0kvSW1Ga2JXbHVJanBzYjJGa0tDSmtNREZmWm5KbGMyaGZj"
    "R0Z6YzNkdmNtUmZhVzV3ZFhRaUtYMHBPd29nSUNBZ2FXWW9jM0JsWXk1cmFXNWtQVDA5SW5CaGMzTjNiM0prSWlZbUtIUjVj"
    "R1Z2WmlCc2IyRmtLQ0prTURGZlpuSmxjMmhmY0dGemMzZHZjbVJmYVc1d2RYUWlLU0U5UFNKemRISnBibWNpZkh3S0lDQWdJ"
    "Q0FnSUNFdlhsdEJMVnBoTFhvd0xUbGZMVjE3TXpBc01USTRmU1F2TG5SbGMzUW9iRzloWkNnaVpEQXhYMlp5WlhOb1gzQmhj"
    "M04zYjNKa1gybHVjSFYwSWlrcEtTa2dld29nSUNBZ0lDQnNZWFJqYUNnaVluSnZkM05sY2w5a1pXTnBjMmx2Ymw5MWJtTnZi"
    "bVpwY20xbFpDSXNSR0YwWlM1dWIzY29LU2s3WVhkaGFYUWdZMnhsWVc1MWNDZ3BPM0psZEhWeWJqc0tJQ0FnSUgwS0lDQWdJ"
    "R0YzWVdsMElHTm9aV05yWldRb0ltSnliM2R6WlhKZmFXNXdkWFFpTEc5d1pYSmhkR2x2Yml4eVBUNTdDaUFnSUNBZ0lHTnZi"
    "bk4wSUhNOWNqOHVjM1J5ZFdOMGRYSmxaRU52Ym5SbGJuUTdDaUFnSUNBZ0lISmxkSFZ5YmlCeVB5NXBjMFZ5Y205eUlUMDlk"
    "SEoxWlNZbVd5SmpiMjVtYVhKdFpXUWlMQ0oxYm5abGNtbG1hV0ZpYkdVaVhTNXBibU5zZFdSbGN5aHpQeTVsWm1abFkzUXBP"
    "d29nSUNBZ2ZTazdJQzh2SUVScGMzQmhkR05vSUdGc2IyNWxJRzVsZG1WeUlHVmhjbTV6SUdGdUlHRndjR3hwWTJGMGFXOXVJ"
    "RzkxZEdOdmJXVWdiM0lnYW05MWNtNWxlU0JqY21Wa2FYUXVDaUFnSUNCemRHOXlaU2dpWkRBeFgyWnlaWE5vWDNCaGMzTjNi"
    "M0prWDJsdWNIVjBJaXh1ZFd4c0tUc0tJQ0FnSUdsbUtISmxZMjl5WkM1bWFYSnpkRjltWVdsc2RYSmxQVDA5Ym5Wc2JDa0tJ"
    "Q0FnSUNBZ1lYZGhhWFFnWTJobFkydGxaQ2dpWW5KdmQzTmxjbDl6Ym1Gd2MyaHZkQ0lzQ2lBZ0lDQWdJQ0FnS0NrOVBuUnZi"
    "Mnh6TG0xamNGOWZZM1ZoWDJSeWFYWmxjbDlmWjJWMFgySnliM2R6WlhKZmMzUmhkR1VvYzI1aGNITm9iM1JCY21kektTd0tJ"
    "Q0FnSUNBZ0lDQnlQVDV3WVdkbEtISXNJbWRsYm1WeWFXTmZZMmhsWTJ0bFpDSXBKaVp3ZFdKc2FXTlFjbVZrYVdOaGRHVW9j"
    "aXh6Y0dWaktTazdDaUFnZlFvZ0lHbG1LSEpsWTI5eVpDNW1hWEp6ZEY5bVlXbHNkWEpsUFQwOWJuVnNiQ2xoZDJGcGRDQndi"
    "MnhzUTI5dWRISnZiR3hsY2lncE93b2dJR2xtS0hKbFkyOXlaQzVtYVhKemRGOW1ZV2xzZFhKbElUMDliblZzYkNsaGQyRnBk"
    "Q0JqYkdWaGJuVndLQ2s3Q2lBZ1pXeHpaU0JwWmloemNHVmpMbXRwYm1ROVBUMGljSEp2ZEdWamRHVmtYMkZtZEdWeUlpa2dl"
    "d29nSUNBZ2NtVmpiM0prTG1Oc1pXRnVkWEJmY21WeGRXVnpkR1ZrWDNkaGJHeGZiWE05UkdGMFpTNXViM2NvS1R0eVpYUmhh"
    "VzRvS1RzS0lDQWdJR0YzWVdsMElHTnNaV0Z1ZFhBb0tUc0tJQ0I5Q24wS1puVnVZM1JwYjI0Z2NIVmliR2xqVUhKbFpHbGpZ"
    "WFJsS0hKbGMzVnNkQ3h6Y0dWaktTQjdDaUFnWTI5dWMzUWdibTlrWlhNOWNtVnpkV3gwUHk1emRISjFZM1IxY21Wa1EyOXVk"
    "R1Z1ZEQ4dVkyOXVkR1Z1ZEY5eVpXWnpPd29nSUM4dklGSnZiM1FnYlhWemRDQndhVzRnYjI1bElIQnlhVzUwWldRZ2NIVmli"
    "R2xqSUdWNGNHVmpkR1ZrSUd4aFltVnNMM0p2YkdVZ1ltVm1iM0psSUhSb1lYUWdZV04wYVc5dUxnb2dJQzh2SUU1dklISmhk"
    "eUJtYVdWc1pDQjJZV3gxWlN3Z1ZWSk1MQ0J6ZFdKcVpXTjBMQ0J4ZFdWeWVTQnZjaUJoY21KcGRISmhjbmtnY21WblpYZ2dj"
    "SEpsWkdsallYUmxJR2x6SUdGc2JHOTNaV1F1Q2lBZ1kyOXVjM1FnY205c1pYTTlXeUpvWldGa2FXNW5JaXdpWW5WMGRHOXVJ"
    "aXdpYzNSaGRHbGpkR1Y0ZENJc0luUmxlSFJpYjNnaVhUc0tJQ0JqYjI1emRDQnNZV0psYkhNOVd5SlZjMlZ5Ym1GdFpTSXNJ"
    "bEJoYzNOM2IzSmtJaXdpUVhWMGFHVnVkR2xqWVhSdmNpQnZjaUJ5WldOdmRtVnllU0JqYjJSbElpd2lVMmxuYmlCcGJpSXND"
    "aUFnSUNBaVRHOWpZV3dnWkdWdGJ5SXNJbEJ5YjNSbFkzUmxaQ0JoY0hCc2FXTmhkR2x2YmlCaFkyTmxjM01pTEFvZ0lDQWdJ"
    "bE5wWjI1bFpDQnBiaTRnVUhKdmRHVmpkR1ZrSUdGd2NHeHBZMkYwYVc5dUlHRmpZMlZ6Y3lCcGN5QmhkbUZwYkdGaWJHVXVJ"
    "bDA3Q2lBZ2NtVjBkWEp1SUVGeWNtRjVMbWx6UVhKeVlYa29ibTlrWlhNcEppWnliMnhsY3k1cGJtTnNkV1JsY3loemNHVmpM"
    "bVY0Y0dWamRHVmtYM0p2YkdVcEppWUtJQ0FnSUd4aFltVnNjeTVwYm1Oc2RXUmxjeWh6Y0dWakxtVjRjR1ZqZEdWa1gyeGhZ"
    "bVZzS1NZbUNpQWdJQ0J1YjJSbGN5NXpiMjFsS0c0OVBtNHVjbTlzWlQwOVBYTndaV011Wlhod1pXTjBaV1JmY205c1pTWW1i"
    "aTV1WVcxbFBUMDljM0JsWXk1bGVIQmxZM1JsWkY5c1lXSmxiQ2s3Q24wS2RISjVJSHNLSUNCcFppaHZkMjR1Y0doaGMyVTlQ"
    "VDBpY0hKbGNHRnlaV1JmWlc1MGNua2lLV0YzWVdsMElHVnVkSEo1S0NrN1pXeHpaU0JoZDJGcGRDQmpiMjUwYVc1MVlYUnBi"
    "MjRvS1RzS2ZTQmpZWFJqYUNCN0NpQWdiR0YwWTJnb0ltTnZiWEJ2YzJWa1gyUmxZMmx6YVc5dVgyVjRZMlZ3ZEdsdmJpSXNS"
    "R0YwWlM1dWIzY29LU2s3Q2lBZ1lYZGhhWFFnWTJ4bFlXNTFjQ2dwT3dwOUNtbG1LSEpsWTI5eVpDNW1hWEp6ZEY5bVlXbHNk"
    "WEpsSVQwOWJuVnNiQ2tnZXdvZ0lIUmxlSFFvZTNKbGMzVnNkRG9pWm1GcGJHVmtJaXhtYVhKemRGOW1ZV2xzZFhKbE9uSmxZ"
    "Mjl5WkM1bWFYSnpkRjltWVdsc2RYSmxMQW9nSUNBZ2RXNWxlSEJsWTNSbFpGOW1ZV2xzZFhKbFgyOWljMlZ5ZG1GMGFXOXVP"
    "bkpsWTI5eVpDNTFibVY0Y0dWamRHVmtYMlpoYVd4MWNtVmZiMkp6WlhKMllYUnBiMjRzQ2lBZ0lDQmthV0ZuYm05emRHbGpY"
    "MlZ5Y205eWN6cHlaV052Y21RdVpHbGhaMjV2YzNScFkxOWxjbkp2Y25Nc1kyeGxZVzUxY0Y5bGNuSnZjbk02Y21WamIzSmtM"
    "bU5zWldGdWRYQmZaWEp5YjNKekxBb2dJQ0FnWm1sdVlXeGZZV0p6Wlc1alpUcHlaV052Y21RdVptbHVZV3hmWVdKelpXNWpa"
    "U3hqYjI1MGNtOXNiR1Z5WDJWNGFYUTZjbVZqYjNKa0xtTnZiblJ5YjJ4c1pYSmZaWGhwZEN3S0lDQWdJSGRvYjJ4bFgyTnNa"
    "V0Z1ZFhCZmQybDBhR2x1TmpCZmNISnZkbVZ1T21aaGJITmxMQW9nSUNBZ2NtVnpiM1Z5WTJWZmNtVnNaV0Z6WlY5d2NtOTJa"
    "VzQ2Y21WamIzSmtMbVpwYm1Gc1gyRmljMlZ1WTJVaFBUMXVkV3hzSmlZS0lDQWdJQ0FnSVhKbFkyOXlaQzVqYkdWaGJuVndY"
    "MlZ5Y205eWN5NXpiMjFsS0hZOVBsc0tJQ0FnSUNBZ0lDQWlZMmhwYkdSZmNtVmhjRjlwYm1OdmJYQnNaWFJsSWl3aWIzZHVa"
    "V1JmY21WemIzVnlZMlZmWVdKelpXNWpaVjkxYm5CeWIzWmxiaUlzQ2lBZ0lDQWdJQ0FnSW05M2JtVmtYM0psWVdSaVlXTnJY"
    "M1Z1WVhaaGFXeGhZbXhsSWl3aWIzZHVaV1JmYkc5allXeGZZMjl0YldGdVpGOTFibXB2YVc1bFpDSmRMbWx1WTJ4MVpHVnpL"
    "SFlwS1gwcE93cDlJR1ZzYzJVZ2V3b2dJSFJsZUhRb2UzSmxjM1ZzZERvaVluSnZkM05sY2w5a1pXTnBjMmx2Ymw5dlluTmxj"
    "blpsWkY5dmJteDVJaXh3WVdkbFgyWnNZV2R6T25KbFkyOXlaQzVzWVhOMFgzQmhaMlZmWm14aFozTS9QMjUxYkd3c0NpQWdJ"
    "Q0JxYjNWeWJtVjVYMk55WldScGREcG1ZV3h6WlN4amJHVmhiblZ3WDJWeWNtOXljenB5WldOdmNtUXVZMnhsWVc1MWNGOWxj"
    "bkp2Y25Nc0NpQWdJQ0J5WlhOdmRYSmpaVjl5Wld4bFlYTmxYM0J5YjNabGJqcHlaV052Y21RdVkyeGxZVzUxY0Y5emRHRnlk"
    "R1ZrUFQwOWRISjFaU1ltY21WamIzSmtMbVpwYm1Gc1gyRmljMlZ1WTJVaFBUMXVkV3hzSmlZS0lDQWdJQ0FnSVhKbFkyOXla"
    "QzVqYkdWaGJuVndYMlZ5Y205eWN5NXpiMjFsS0hZOVBsc0tJQ0FnSUNBZ0lDQWlZMmhwYkdSZmNtVmhjRjlwYm1OdmJYQnNa"
    "WFJsSWl3aWIzZHVaV1JmY21WemIzVnlZMlZmWVdKelpXNWpaVjkxYm5CeWIzWmxiaUlzQ2lBZ0lDQWdJQ0FnSW05M2JtVmtY"
    "M0psWVdSaVlXTnJYM1Z1WVhaaGFXeGhZbXhsSWl3aWIzZHVaV1JmYkc5allXeGZZMjl0YldGdVpGOTFibXB2YVc1bFpDSmRM"
    "bWx1WTJ4MVpHVnpLSFlwS1N3S0lDQWdJSGRvYjJ4bFgyTnNaV0Z1ZFhCZmQybDBhR2x1TmpCZmNISnZkbVZ1T21aaGJITmxm"
    "U2s3Q24wSyIsCiAgImRpZmZfYjY0IjogIkxTMHRJR2x0YlhWMFlXSnNaUzFqTldRMFpERTNNeTFqWld4c0xXUTNNamM0Tnpn"
    "d0Npc3JLeUJFUlZOSlIwNHRiMjVzZVMxR01pMUdNeTFHTkFwQVFDQXRNVE01TERjZ0t6RXpPU3c1SUVCQUNpQWdJQ0FnSUNC"
    "dlluTmxjblpoZEdsdmJpNW9aV3h3WlhKZlpYaHBkRDEwZVhCbGIyWWdaWFpsYm5RdVpYaHBkRDA5UFNKdWRXMWlaWElpUDJW"
    "MlpXNTBMbVY0YVhRNmJuVnNiRHNLSUNBZ0lDQWdJSEpsZEdGcGJpZ3BPd29nSUNBZ0lDQWdhV1lvWlhabGJuUXVaWGhwZENF"
    "OVBUQXBiR0YwWTJnb0ltaGxiSEJsY2w5bVlXbHNaV1FpTEhKbFkyVnBkbVZrVjJGc2JDazdDaTBnSUNBZ0lDQmxiSE5sSUds"
    "bUtDRmpiR1ZoYm5Wd1UzUmhjblJsWkNZbWNtVmpiM0prTG5CeWIzUmxZM1JsWkY5aFpuUmxjbDl3WVdkbElUMDlkSEoxWlNr"
    "S0t5QWdJQ0FnSUdWc2MyVWdhV1lvSVdOc1pXRnVkWEJUZEdGeWRHVmtKaVp5WldOdmNtUXVjSEp2ZEdWamRHVmtYMkZtZEdW"
    "eVgzQmhaMlVoUFQxMGNuVmxKaVlLS3lBZ0lDQWdJQ0FnSUNBZ0lDQWdJU2h2ZDI0dWNHaGhjMlU5UFQwaVluSnZkM05sY2w5"
    "a1pXTnBjMmx2YmlJbUpnb3JJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lHOTNiaTV1WlhoMFgyUmxZMmx6YVc5dVB5NXJhVzVrUFQw"
    "OUluQnliM1JsWTNSbFpGOWhablJsY2lJcEtRb2dJQ0FnSUNBZ0lDQnNZWFJqYUNnaWFHVnNjR1Z5WDJOdmJYQnNaWFJsWkY5"
    "aVpXWnZjbVZmWVhCd1gyTm9aV05yY0c5cGJuUWlMSEpsWTJWcGRtVmtWMkZzYkNrN0NpQWdJQ0FnZlFvZ0lDQWdJR2xtS0dW"
    "MlpXNTBMbVpwZUhSMWNtVmZabWx1YVhOb1pXUTlQVDEwY25WbEtTQjdDa0JBSUMweE9Ea3NOaUFyTVRreExEY2dRRUFLSUNB"
    "Z2NtVjBZV2x1S0NrN0NpQjlDaUJoYzNsdVl5Qm1kVzVqZEdsdmJpQmpiR1ZoYm5Wd0tDa2dld29ySUNCemRHOXlaU2dpWkRB"
    "eFgyWnlaWE5vWDNCaGMzTjNiM0prWDJsdWNIVjBJaXh1ZFd4c0tUc2dMeThnUTJ4bFlYSWdZbVZtYjNKbElHRnVlU0JqYkdW"
    "aGJuVndJR0YzWVdsMExnb2dJQ0JwWmloamJHVmhiblZ3VTNSaGNuUmxaQ2x5WlhSMWNtNDdDaUFnSUdOc1pXRnVkWEJUZEdG"
    "eWRHVmtQWFJ5ZFdVN2NtVmpiM0prTG1Oc1pXRnVkWEJmYzNSaGNuUmxaRDEwY25WbE8zSmxkR0ZwYmlncE93b2dJQ0IwY25r"
    "Z2V3cEFRQ0F0TkRFMUxEY2dLelF4T0N3M0lFQkFDaUFnSUNBZ0lDQmpiMjV6ZENCelBYSS9Mbk4wY25WamRIVnlaV1JEYjI1"
    "MFpXNTBPd29nSUNBZ0lDQWdjbVYwZFhKdUlISS9MbWx6UlhKeWIzSWhQVDEwY25WbEppWmJJbU52Ym1acGNtMWxaQ0lzSW5W"
    "dWRtVnlhV1pwWVdKc1pTSmRMbWx1WTJ4MVpHVnpLSE0vTG1WbVptVmpkQ2s3Q2lBZ0lDQWdmU2s3SUM4dklFUnBjM0JoZEdO"
    "b0lHRnNiMjVsSUc1bGRtVnlJR1ZoY201eklHRnVJR0Z3Y0d4cFkyRjBhVzl1SUc5MWRHTnZiV1VnYjNJZ2FtOTFjbTVsZVNC"
    "amNtVmthWFF1Q2kwZ0lDQWdjM1J2Y21Vb0ltUXdNVjltY21WemFGOXdZWE56ZDI5eVpGOXBibkIxZENJc2JuVnNiQ2s3Q2lz"
    "Z0lDQWdhV1lvYzNCbFl5NXJhVzVrUFQwOUluQmhjM04zYjNKa0lpbHpkRzl5WlNnaVpEQXhYMlp5WlhOb1gzQmhjM04zYjNK"
    "a1gybHVjSFYwSWl4dWRXeHNLVHNLSUNBZ0lDQnBaaWh5WldOdmNtUXVabWx5YzNSZlptRnBiSFZ5WlQwOVBXNTFiR3dwQ2lB"
    "Z0lDQWdJQ0JoZDJGcGRDQmphR1ZqYTJWa0tDSmljbTkzYzJWeVgzTnVZWEJ6YUc5MElpd0tJQ0FnSUNBZ0lDQWdLQ2s5UG5S"
    "dmIyeHpMbTFqY0Y5ZlkzVmhYMlJ5YVhabGNsOWZaMlYwWDJKeWIzZHpaWEpmYzNSaGRHVW9jMjVoY0hOb2IzUkJjbWR6S1N3"
    "S1FFQWdMVFEwTWl3MklDczBORFVzTWpFZ1FFQUtJSDBLSUhSeWVTQjdDaUFnSUdsbUtHOTNiaTV3YUdGelpUMDlQU0p3Y21W"
    "d1lYSmxaRjlsYm5SeWVTSXBZWGRoYVhRZ1pXNTBjbmtvS1R0bGJITmxJR0YzWVdsMElHTnZiblJwYm5WaGRHbHZiaWdwT3dv"
    "cklDQXZMeUJFYnlCdWIzUWdZMkZ5Y25rZ2RXNTJZV3hwWkdGMFpXUWdjR0Z5ZEdsaGJDQmpiMjUwY205c2JHVnlJSFJsZUhR"
    "Z1lXTnliM056SUhSb2FYTWdZMlZzYkNCaWIzVnVaR0Z5ZVM0S0t5QWdkMmhwYkdVb1kyOXVkSEp2Ykd4bGNrSjFabVpsY2k1"
    "c1pXNW5kR2crTUNZbWNtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVTlQVDF1ZFd4c0tTQjdDaXNnSUNBZ1kyOXVjM1FnWW1W"
    "bWIzSmxVRzlzYkZkaGJHdzlSR0YwWlM1dWIzY29LVHNLS3lBZ0lDQnBaaWhpWldadmNtVlFiMnhzVjJGc2JENDliM2R1TG5O"
    "MFlYSjBYMlZ3YjJOb1gyMXpLemcwTURBd01Da2dld29ySUNBZ0lDQWdiR0YwWTJnb0ltSnliM2R6WlhKZllXTjBhWFpsWDJS"
    "bFlXUnNhVzVsSWl4aVpXWnZjbVZRYjJ4c1YyRnNiQ2s3WW5KbFlXczdDaXNnSUNBZ2ZRb3JJQ0FnSUdsbUtHTnZiblJ5YjJ4"
    "c1pYSktiMmx1WldSOGZDRmpiMjUwY205c2JHVnlVRzlzYkVGMllXbHNZV0pzWlNrZ2V3b3JJQ0FnSUNBZ2JHRjBZMmdvSW1O"
    "dmJuUnliMnhzWlhKZmIySnpaWEoyWVhScGIyNWZhVzUyWVd4cFpDSXNZbVZtYjNKbFVHOXNiRmRoYkd3cE8ySnlaV0ZyT3dv"
    "cklDQWdJSDBLS3lBZ0lDQmhkMkZwZENCd2IyeHNRMjl1ZEhKdmJHeGxjaWdwT3lBdkx5QlRZVzFsSUc5M2JtVmtJSE5sYzNO"
    "cGIyNDdJRzlpYzJWeWRtVnlJSEpsZEdGcGJuTWdjbVZqWldsd2RDQm1hWEp6ZEM0S0t5QWdJQ0JqYjI1emRDQmhablJsY2xC"
    "dmJHeFhZV3hzUFVSaGRHVXVibTkzS0NrN0Npc2dJQ0FnYVdZb1lXWjBaWEpRYjJ4c1YyRnNiRDQ5YjNkdUxuTjBZWEowWDJW"
    "d2IyTm9YMjF6S3pnME1EQXdNQ2tLS3lBZ0lDQWdJR3hoZEdOb0tDSmljbTkzYzJWeVgyRmpkR2wyWlY5a1pXRmtiR2x1WlNJ"
    "c1lXWjBaWEpRYjJ4c1YyRnNiQ2s3Q2lzZ0lIMEtLeUFnYVdZb2NtVmpiM0prTG1acGNuTjBYMlpoYVd4MWNtVWhQVDF1ZFd4"
    "c0tXRjNZV2wwSUdOc1pXRnVkWEFvS1RzS0lIMGdZMkYwWTJnZ2V3b2dJQ0JzWVhSamFDZ2lZMjl0Y0c5elpXUmZaR1ZqYVhO"
    "cGIyNWZaWGhqWlhCMGFXOXVJaXhFWVhSbExtNXZkeWdwS1RzS0lDQWdZWGRoYVhRZ1kyeGxZVzUxY0NncE93bz0iLAogICJj"
    "YW5kaWRhdGVfc2hhMjU2IjogIjdhYjIzMDE5ZTgzODUxMmE4NDYwMGQ0MDY5ZmVmMmU0MTliOWJiN2YwN2ZmNWFhYTJjNzQ3"
    "N2FmOTU2ZjljYTgiLAogICJiYXNlbGluZV9zaGEyNTYiOiAiZDcyNzg3ODAwOWI5NTVlNmM2ZTMwOWZmYTkwZmUzMGMzNjQ0"
    "MDg5YjRhMGZiN2ViYmU0ZWJkNzI2NGZkM2Q5ZCIsCiAgImRpZmZfc2hhMjU2IjogIjM5NTk0ODY1NzA3NWM3ZmIyNzUwM2I1"
    "MDdkMTgzMDMwODU3NTVhZjllNTMxMzAzMTk3Yjk2NGFkYzdlY2YxNDQiLAogICJsb2dpY19zaGEyNTYiOiAiNjM5ZGY3ZjE1"
    "YTFmODEzNWI1N2VjYmFjYjE1MDYyY2RjNzVkYTEwMDUzMGU0YzA2MmU4NTZlN2VkMmE4NzA1NCIsCiAgImVkaXRzIjogWwog"
    "ICAgewogICAgICAiaWQiOiAiRjItY2xlYW51cCIsCiAgICAgICJvbGQiOiAiICBpZihjbGVhbnVwU3RhcnRlZClyZXR1cm47"
    "XG4gIGNsZWFudXBTdGFydGVkPXRydWU7cmVjb3JkLmNsZWFudXBfc3RhcnRlZD10cnVlO3JldGFpbigpOyIsCiAgICAgICJu"
    "ZXh0IjogIiAgc3RvcmUoXCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXRcIixudWxsKTsgLy8gQ2xlYXIgYmVmb3JlIGFueSBj"
    "bGVhbnVwIGF3YWl0LlxuICBpZihjbGVhbnVwU3RhcnRlZClyZXR1cm47XG4gIGNsZWFudXBTdGFydGVkPXRydWU7cmVjb3Jk"
    "LmNsZWFudXBfc3RhcnRlZD10cnVlO3JldGFpbigpOyIKICAgIH0sCiAgICB7CiAgICAgICJpZCI6ICJGMi1wYXNzd29yZCIs"
    "CiAgICAgICJvbGQiOiAiICAgIHN0b3JlKFwiZDAxX2ZyZXNoX3Bhc3N3b3JkX2lucHV0XCIsbnVsbCk7XG4gICAgaWYocmVj"
    "b3JkLmZpcnN0X2ZhaWx1cmU9PT1udWxsKSIsCiAgICAgICJuZXh0IjogIiAgICBpZihzcGVjLmtpbmQ9PT1cInBhc3N3b3Jk"
    "XCIpc3RvcmUoXCJkMDFfZnJlc2hfcGFzc3dvcmRfaW5wdXRcIixudWxsKTtcbiAgICBpZihyZWNvcmQuZmlyc3RfZmFpbHVy"
    "ZT09PW51bGwpIgogICAgfSwKICAgIHsKICAgICAgImlkIjogIkYzLWZpbmFsLXNuYXBzaG90IiwKICAgICAgIm9sZCI6ICIg"
    "ICAgICBlbHNlIGlmKCFjbGVhbnVwU3RhcnRlZCYmcmVjb3JkLnByb3RlY3RlZF9hZnRlcl9wYWdlIT09dHJ1ZSlcbiAgICAg"
    "ICAgbGF0Y2goXCJoZWxwZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludFwiLHJlY2VpdmVkV2FsbCk7IiwKICAg"
    "ICAgIm5leHQiOiAiICAgICAgZWxzZSBpZighY2xlYW51cFN0YXJ0ZWQmJnJlY29yZC5wcm90ZWN0ZWRfYWZ0ZXJfcGFnZSE9"
    "PXRydWUmJlxuICAgICAgICAgICAgICAhKG93bi5waGFzZT09PVwiYnJvd3Nlcl9kZWNpc2lvblwiJiZcbiAgICAgICAgICAg"
    "ICAgICBvd24ubmV4dF9kZWNpc2lvbj8ua2luZD09PVwicHJvdGVjdGVkX2FmdGVyXCIpKVxuICAgICAgICBsYXRjaChcImhl"
    "bHBlcl9jb21wbGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50XCIscmVjZWl2ZWRXYWxsKTsiCiAgICB9LAogICAgewogICAg"
    "ICAiaWQiOiAiRjQtZHJhaW4iLAogICAgICAib2xkIjogIiAgaWYob3duLnBoYXNlPT09XCJwcmVwYXJlZF9lbnRyeVwiKWF3"
    "YWl0IGVudHJ5KCk7ZWxzZSBhd2FpdCBjb250aW51YXRpb24oKTtcbn0gY2F0Y2ggeyIsCiAgICAgICJuZXh0IjogIiAgaWYo"
    "b3duLnBoYXNlPT09XCJwcmVwYXJlZF9lbnRyeVwiKWF3YWl0IGVudHJ5KCk7ZWxzZSBhd2FpdCBjb250aW51YXRpb24oKTtc"
    "biAgLy8gRG8gbm90IGNhcnJ5IHVudmFsaWRhdGVkIHBhcnRpYWwgY29udHJvbGxlciB0ZXh0IGFjcm9zcyB0aGlzIGNlbGwg"
    "Ym91bmRhcnkuXG4gIHdoaWxlKGNvbnRyb2xsZXJCdWZmZXIubGVuZ3RoPjAmJnJlY29yZC5maXJzdF9mYWlsdXJlPT09bnVs"
    "bCkge1xuICAgIGNvbnN0IGJlZm9yZVBvbGxXYWxsPURhdGUubm93KCk7XG4gICAgaWYoYmVmb3JlUG9sbFdhbGw+PW93bi5z"
    "dGFydF9lcG9jaF9tcys4NDAwMDApIHtcbiAgICAgIGxhdGNoKFwiYnJvd3Nlcl9hY3RpdmVfZGVhZGxpbmVcIixiZWZvcmVQ"
    "b2xsV2FsbCk7YnJlYWs7XG4gICAgfVxuICAgIGlmKGNvbnRyb2xsZXJKb2luZWR8fCFjb250cm9sbGVyUG9sbEF2YWlsYWJs"
    "ZSkge1xuICAgICAgbGF0Y2goXCJjb250cm9sbGVyX29ic2VydmF0aW9uX2ludmFsaWRcIixiZWZvcmVQb2xsV2FsbCk7YnJl"
    "YWs7XG4gICAgfVxuICAgIGF3YWl0IHBvbGxDb250cm9sbGVyKCk7IC8vIFNhbWUgb3duZWQgc2Vzc2lvbjsgb2JzZXJ2ZXIg"
    "cmV0YWlucyByZWNlaXB0IGZpcnN0LlxuICAgIGNvbnN0IGFmdGVyUG9sbFdhbGw9RGF0ZS5ub3coKTtcbiAgICBpZihhZnRl"
    "clBvbGxXYWxsPj1vd24uc3RhcnRfZXBvY2hfbXMrODQwMDAwKVxuICAgICAgbGF0Y2goXCJicm93c2VyX2FjdGl2ZV9kZWFk"
    "bGluZVwiLGFmdGVyUG9sbFdhbGwpO1xuICB9XG4gIGlmKHJlY29yZC5maXJzdF9mYWlsdXJlIT09bnVsbClhd2FpdCBjbGVh"
    "bnVwKCk7XG59IGNhdGNoIHsiCiAgICB9CiAgXSwKICAiY29sbGVjdG9ycyI6IHsKICAgICJDTE9DSyI6IHsKICAgICAgImI2"
    "NCI6ICJhVzF3YjNKMElHcHpiMjRzZEdsdFpRcHdjbWx1ZENocWMyOXVMbVIxYlhCektIc25iVzl1YjNSdmJtbGpYMjV6Snpw"
    "emRISW9kR2x0WlM1dGIyNXZkRzl1YVdOZmJuTW9LU2tzSjNkaGJHeGZaWEJ2WTJoZmJuTW5Pbk4wY2loMGFXMWxMblJwYldW"
    "ZmJuTW9LU2w5S1NrSyIsCiAgICAgICJieXRlcyI6IDExNCwKICAgICAgInNoYTI1NiI6ICJjZmIzZjMxNTRjYjc3ODM4ODcy"
    "NDcyOGI2ZjdjYjgwNGJkN2UxNDI4NDRkZWQ3YzA0NGFmYzhkZDMxMzVhYjRkIgogICAgfSwKICAgICJTVE9QIjogewogICAg"
    "ICAiYjY0IjogImFXMXdiM0owSUdwemIyNHNiM01zY0dGMGFHeHBZaXh6ZEdGMExITjVjeXgwYVcxbENtTTlhbk52Ymk1c2Iy"
    "RmtjeWh6ZVhNdVlYSm5kbHN4WFNrS1kyeHZZMnM5ZXlkdGIyNXZkRzl1YVdOZmJuTW5Pbk4wY2loMGFXMWxMbTF2Ym05MGIy"
    "NXBZMTl1Y3lncEtTd25kMkZzYkY5bGNHOWphRjl1Y3ljNmMzUnlLSFJwYldVdWRHbHRaVjl1Y3lncEtYMEtjbVZqYjNKa1BY"
    "c25jMk5vWlcxaEp6b25jbWxoZFhSb0xtUXdNUzFtYVhKemRDMXZZbk5sY25aaGRHbHZiaTkyTVNjc0oyWnBjbk4wWDJaaGFX"
    "eDFjbVVuT21OYkoyWnBjbk4wWDJaaGFXeDFjbVVuWFN3blptbHljM1JmYjJKelpYSjJZWFJwYjI1ZmQyRnNiRjl0Y3ljNlkx"
    "c25abWx5YzNSZmIySnpaWEoyWVhScGIyNWZkMkZzYkY5dGN5ZGRMQ2RtYVhKemRGOWxkbVZ1ZEY5d2NtOTJaVzRuT2taaGJI"
    "TmxMQ2RqYkc5amF5YzZZMnh2WTJ0OUNtVjJaVzUwWDNOMFlYUmxQU2QzY21sMFpWOTFibU52Ym1acGNtMWxaQ2NLZEhKNU9n"
    "b2dJQ0FnWm1ROWIzTXViM0JsYmlod1lYUm9iR2xpTGxCaGRHZ29ZMXNuWlhabGJuUmZiM1YwSjEwcExHOXpMazlmVjFKUFRr"
    "eFpmRzl6TGs5ZlExSkZRVlI4YjNNdVQxOUZXRU5NZkc5ekxrOWZUazlHVDB4TVQxY3NNRzgyTURBcENpQWdJQ0IzYVhSb0lH"
    "OXpMbVprYjNCbGJpaG1aQ3duZHljc1pXNWpiMlJwYm1jOUoyRnpZMmxwSnlrZ1lYTWdaam9LSUNBZ0lDQWdJQ0JtTG5keWFY"
    "UmxLR3B6YjI0dVpIVnRjSE1vY21WamIzSmtMSE52Y25SZmEyVjVjejFVY25WbEtTc25YRzRuS1R0bUxtWnNkWE5vS0NrN2Iz"
    "TXVabk41Ym1Nb1ppNW1hV3hsYm04b0tTa0tJQ0FnSUdWMlpXNTBYM04wWVhSbFBTZDNjbWwwZEdWdUp3cGxlR05sY0hRZ1Qx"
    "TkZjbkp2Y2pwd1lYTnpDaU1nUVhSMFpXMXdkQ0JqYkc5amF5QndaWEp6YVhOMFpXNWpaU0JDUlVaUFVrVWdiV0Z5YTJWeUwz"
    "TjBZWFJsTDJKMVpHZGxkQ0JqYjIxd1lYSnBjMjl1Y3k0S0l5QkJJRzFsZEdGa1lYUmhJR1Z5Y205eUlHUnZaWE1nYm05MElI"
    "QnlaWFpsYm5RZ2RHaGxJRzl5YVdkcGJtRnNJR1Z6YzJWdWRHbGhiQ0J6ZEc5d0lIQnliM1J2WTI5c0xncHNZV0k5Y0dGMGFH"
    "eHBZaTVRWVhSb0tHTmJKMnhoWWlkZEtUdHRZWEpyWlhKZmMzUmhkR1U5SjJ4aFlsOWhZbk5sYm5RbkNuUnllVG9LSUNBZ0lH"
    "bG1JR3hoWWk1bGVHbHpkSE1vS1RvS0lDQWdJQ0FnSUNCcGJtWnZQV3hoWWk1c2MzUmhkQ2dwQ2lBZ0lDQWdJQ0FnYVdZZ2Jt"
    "OTBJSE4wWVhRdVUxOUpVMFJKVWlocGJtWnZMbk4wWDIxdlpHVXBJRzl5SUhOMFlYUXVVMTlKVFU5RVJTaHBibVp2TG5OMFgy"
    "MXZaR1VwSVQwd2J6Y3dNQ0J2Y2lCcGJtWnZMbk4wWDNWcFpDRTliM011WjJWMGRXbGtLQ2s2Q2lBZ0lDQWdJQ0FnSUNBZ0lH"
    "MWhjbXRsY2w5emRHRjBaVDBuYjNkdVpYSnphR2x3WDNWdWEyNXZkMjRuQ2lBZ0lDQWdJQ0FnWld4elpUb0tJQ0FnSUNBZ0lD"
    "QWdJQ0FnYldGeWEyVnlYM04wWVhSbFBTZHpkRzl3WDNKbGNYVmxjM1JsWkNjS0lDQWdJQ0FnSUNBZ0lDQWdabTl5SUc1aGJX"
    "VWdhVzRnS0NnbmRXa3RabUZwYkhWeVpTY3NKM04wYjNBbktTQnBaaUJqV3lkbWFYSnpkRjltWVdsc2RYSmxKMTBnYVhNZ2Jt"
    "OTBJRTV2Ym1VZ1pXeHpaU0FvSjNOMGIzQW5MQ2twT2dvZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnZEhKNU9nb2dJQ0FnSUNBZ0lD"
    "QWdJQ0FnSUNBZ0lDQWdJR1prUFc5ekxtOXdaVzRvYkdGaUwyNWhiV1VzYjNNdVQxOVhVazlPVEZsOGIzTXVUMTlEVWtWQlZI"
    "eHZjeTVQWDBWWVEweDhiM011VDE5T1QwWlBURXhQVnl3d2J6WXdNQ2tLSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNCdmN5"
    "NWpiRzl6WlNobVpDa0tJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lHVjRZMlZ3ZENCR2FXeGxUbTkwUm05MWJtUkZjbkp2Y2pwdFlY"
    "SnJaWEpmYzNSaGRHVTlKMnhoWWw5aFluTmxiblFuQ2lBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0JsZUdObGNIUWdSbWxzWlVWNGFY"
    "TjBjMFZ5Y205eU9uQmhjM01LWlhoalpYQjBJRTlUUlhKeWIzSTZiV0Z5YTJWeVgzTjBZWFJsUFNkemRHOXdYM1Z1WTI5dVpt"
    "bHliV1ZrSndwd2NtbHVkQ2hxYzI5dUxtUjFiWEJ6S0hzblkyeHZZMnNuT21Oc2IyTnJMQ2RsZG1WdWRGOXpkR0YwWlNjNlpY"
    "WmxiblJmYzNSaGRHVXNKMjFoY210bGNsOXpkR0YwWlNjNmJXRnlhMlZ5WDNOMFlYUmxmU2twQ2c9PSIsCiAgICAgICJieXRl"
    "cyI6IDE2MDAsCiAgICAgICJzaGEyNTYiOiAiNWRlNTI0NDQ0MGIxMzQ2ODliZWRmYjEyMzVlODA1YzY1YThiNzI2ZTc1M2Ni"
    "ZGFjMWQwODMxZWYxYzgzZDA3MSIKICAgIH0sCiAgICAiUkVBREJBQ0siOiB7CiAgICAgICJiNjQiOiAiYVcxd2IzSjBJR3B6"
    "YjI0c2IzTXNjR0YwYUd4cFlpeHpkR0YwTEhOMVluQnliMk5sYzNNc2MzbHpMSFJwYldVS1l6MXFjMjl1TG14dllXUnpLSE41"
    "Y3k1aGNtZDJXekZkS1FwamFHbHNaSEpsYmoxT2IyNWxPMmhsYkhCbGNsOXdhV1E5WTFzbmFHVnNjR1Z5WDNCcFpDZGRPMjkx"
    "ZEdWeVgyOWljMlZ5ZG1GMGFXOXVQVTV2Ym1VN2NISnZhbVZqZEdsdmJqMG5kVzVoZG1GcGJHRmliR1VuQ21SbFppQm1hVzVw"
    "ZEdWZmIySnpaWEoyWVhScGIyNG9kbUZzZFdVcE9nb2dJQ0FnYVdZZ2RIbHdaU2gyWVd4MVpTa2dhWE1nYm05MElHUnBZM1Fn"
    "YjNJZ2MyVjBLSFpoYkhWbEtTRTlJSHNuYjJKelpYSjJaV1FuTENka2FXRm5ibTl6ZEdsakozMGdiM0lnZEhsd1pTaDJZV3gx"
    "WlZzbmIySnpaWEoyWldRblhTa2dhWE1nYm05MElHSnZiMnc2Q2lBZ0lDQWdJQ0FnY21WMGRYSnVJRTV2Ym1VS0lDQWdJR1Jw"
    "WVdkdWIzTjBhV005ZG1Gc2RXVmJKMlJwWVdkdWIzTjBhV01uWFFvZ0lDQWdhV1lnWkdsaFoyNXZjM1JwWXlCcGN5Qk9iMjVs"
    "T2dvZ0lDQWdJQ0FnSUhKbGRIVnliaUI3SjI5aWMyVnlkbVZrSnpwMllXeDFaVnNuYjJKelpYSjJaV1FuWFN3blpHbGhaMjV2"
    "YzNScFl5YzZUbTl1WlgwS0lDQWdJR2xtSUc1dmRDQjJZV3gxWlZzbmIySnpaWEoyWldRblhTQnZjaUIwZVhCbEtHUnBZV2R1"
    "YjNOMGFXTXBJR2x6SUc1dmRDQmthV04wSUc5eUlITmxkQ2hrYVdGbmJtOXpkR2xqS1NFOUlIc25jMmwwWlNjc0oyVjRZMlZ3"
    "ZEdsdmJsOWpiR0Z6Y3ljc0oyOTNibDltZFc1amRHbHZiaWNzSjI5M2JsOXNhVzVsSjMwNkNpQWdJQ0FnSUNBZ2NtVjBkWEp1"
    "SUU1dmJtVUtJQ0FnSUhOcGRHVnpQU2duYUdGdVpHeGxjaWNzSjNObGNuWmxjaWNzSjIxaGFXNG5LUW9nSUNBZ2EybHVaSE05"
    "S0NkQmRIUnlhV0oxZEdWRmNuSnZjaWNzSjFSNWNHVkZjbkp2Y2ljc0oxWmhiSFZsUlhKeWIzSW5MQ2RMWlhsRmNuSnZjaWNz"
    "SjA5VFJYSnliM0luTENkQ2NtOXJaVzVRYVhCbFJYSnliM0luTENkRGIyNXVaV04wYVc5dVVtVnpaWFJGY25KdmNpY3NKMVJw"
    "YldWdmRYUkZjbkp2Y2ljc0oyOTBhR1Z5SnlrS0lDQWdJR1oxYm1OMGFXOXVjejBvSjBobFlXUmxjbEpsWVdSbGNpNXlaV0Zr"
    "YkdsdVpTY3NKMFJsYlc5VFpYSjJaWEl1Y0hKdlkyVnpjMTl5WlhGMVpYTjBKeXduUkdWdGJ5NWlaV2RwYmljc0owUmxiVzh1"
    "WTJGc2JHSmhZMnNuTENkRVpXMXZMbWx1ZG05clpTY3NKMGhoYm1Sc1pYSXVhR0Z1Wkd4bFgyOXVaVjl5WlhGMVpYTjBKeXdu"
    "U0dGdVpHeGxjaTV6Wlc1a1gyVnljbTl5Snl3blNHRnVaR3hsY2k1blpYUW5MQ2RJWVc1a2JHVnlMbkpsY0d4NUp5d25iV0Zw"
    "YmljcENpQWdJQ0J6YVhSbExHdHBibVFzWm5WdVkzUnBiMjRzYkdsdVpUMG9aR2xoWjI1dmMzUnBZMXRyWFNCbWIzSWdheUJw"
    "YmlBb0ozTnBkR1VuTENkbGVHTmxjSFJwYjI1ZlkyeGhjM01uTENkdmQyNWZablZ1WTNScGIyNG5MQ2R2ZDI1ZmJHbHVaU2Nw"
    "S1FvZ0lDQWdhV1lnZEhsd1pTaHphWFJsS1NCcGN5QnViM1FnYzNSeUlHOXlJSE5wZEdVZ2JtOTBJR2x1SUhOcGRHVnpJRzl5"
    "SUhSNWNHVW9hMmx1WkNrZ2FYTWdibTkwSUhOMGNpQnZjaUJyYVc1a0lHNXZkQ0JwYmlCcmFXNWtjem9LSUNBZ0lDQWdJQ0J5"
    "WlhSMWNtNGdUbTl1WlFvZ0lDQWdhV1lnYm05MElDZ29ablZ1WTNScGIyNGdhWE1nVG05dVpTQmhibVFnYkdsdVpTQnBjeUJP"
    "YjI1bEtTQnZjaUFvZEhsd1pTaG1kVzVqZEdsdmJpa2dhWE1nYzNSeUlHRnVaQ0JtZFc1amRHbHZiaUJwYmlCbWRXNWpkR2x2"
    "Ym5NZ1lXNWtJSFI1Y0dVb2JHbHVaU2tnYVhNZ2FXNTBJR0Z1WkNBeFBEMXNhVzVsUEQweE1ESTBLU2s2Q2lBZ0lDQWdJQ0Fn"
    "Y21WMGRYSnVJRTV2Ym1VS0lDQWdJSEpsZEhWeWJpQjdKMjlpYzJWeWRtVmtKenBVY25WbExDZGthV0ZuYm05emRHbGpKenA3"
    "SjNOcGRHVW5Pbk5wZEdVc0oyVjRZMlZ3ZEdsdmJsOWpiR0Z6Y3ljNmEybHVaQ3duYjNkdVgyWjFibU4wYVc5dUp6cG1kVzVq"
    "ZEdsdmJpd25iM2R1WDJ4cGJtVW5PbXhwYm1WOWZRcHZkWFJsY2oxd1lYUm9iR2xpTGxCaGRHZ29ZMXNuYjNWMFpYSmZiM1Yw"
    "SjEwcENtbG1JRzkxZEdWeUxtbHpYMlpwYkdVb0tUb0tJQ0FnSUdaa1BXOXpMbTl3Wlc0b2IzVjBaWElzYjNNdVQxOVNSRTlP"
    "VEZsOGIzTXVUMTlPVDBaUFRFeFBWM3h2Y3k1UFgwNVBUa0pNVDBOTEtRb2dJQ0FnZEhKNU9nb2dJQ0FnSUNBZ0lHbHVabTg5"
    "YjNNdVpuTjBZWFFvWm1RcENpQWdJQ0FnSUNBZ2FXWWdibTkwSUNoemRHRjBMbE5mU1ZOU1JVY29hVzVtYnk1emRGOXRiMlJs"
    "S1NCaGJtUWdhVzVtYnk1emRGOTFhV1E5UFc5ekxtZGxkSFZwWkNncElHRnVaQ0J6ZEdGMExsTmZTVTFQUkVVb2FXNW1ieTV6"
    "ZEY5dGIyUmxLVDA5TUc4Mk1EQWdZVzVrSURBOGFXNW1ieTV6ZEY5emFYcGxQRDB5TmpJeE5EUXBPZ29nSUNBZ0lDQWdJQ0Fn"
    "SUNCeVlXbHpaU0JXWVd4MVpVVnljbTl5S0NkbWFYaGxaRjl2ZFhSbGNsOWxkbWxrWlc1alpWOXBiblpoYkdsa0p5a0tJQ0Fn"
    "SUNBZ0lDQjNhWFJvSUc5ekxtWmtiM0JsYmlobVpDd25jbUluTEdOc2IzTmxabVE5Um1Gc2MyVXBJR0Z6SUhOdmRYSmpaVHB5"
    "WVhkZllubDBaWE05YzI5MWNtTmxMbkpsWVdRb01qWXlNVFExS1FvZ0lDQWdJQ0FnSUdsbUlHNXZkQ0F3UEd4bGJpaHlZWGRm"
    "WW5sMFpYTXBQRDB5TmpJeE5EUTZjbUZwYzJVZ1ZtRnNkV1ZGY25KdmNpZ25abWw0WldSZmIzVjBaWEpmWlhacFpHVnVZMlZm"
    "YVc1MllXeHBaQ2NwQ2lBZ0lDQWdJQ0FnWkdGMFlUMXFjMjl1TG14dllXUnpLSEpoZDE5aWVYUmxjeWtLSUNBZ0lHWnBibUZz"
    "YkhrNmIzTXVZMnh2YzJVb1ptUXBDaUFnSUNCdmRYUmxjbDl2WW5ObGNuWmhkR2x2YmoxbWFXNXBkR1ZmYjJKelpYSjJZWFJw"
    "YjI0b1pHRjBZUzVuWlhRb0ozVnVaWGh3WldOMFpXUmZabUZwYkhWeVpWOXZZbk5sY25aaGRHbHZiaWNwS1FvZ0lDQWdjSEp2"
    "YW1WamRHbHZiajFrWVhSaExtZGxkQ2duZFc1bGVIQmxZM1JsWkY5dlluTmxjblpoZEdsdmJsOXdjbTlxWldOMGFXOXVKeWtL"
    "SUNBZ0lHbG1JSEJ5YjJwbFkzUnBiMjRnYm05MElHbHVJQ2duZFc1aGRtRnBiR0ZpYkdVbkxDZDJZV3hwWkNjc0oybHVkbUZz"
    "YVdRbktUcHdjbTlxWldOMGFXOXVQU2RwYm5aaGJHbGtKd29nSUNBZ2FXWWdjSEp2YW1WamRHbHZiajA5SjNaaGJHbGtKeUJo"
    "Ym1RZ2IzVjBaWEpmYjJKelpYSjJZWFJwYjI0Z2FYTWdUbTl1WlRwd2NtOXFaV04wYVc5dVBTZHBiblpoYkdsa0p3b2dJQ0Fn"
    "WVd4c2IzZGxaRDE3SjNkb2IyRnRhU2NzSjJScGMyTnZkbVZ5ZVNjc0oyTnZibVpwWkdWdWRHbGhiRjlqYkdsbGJuUmZZM0ps"
    "WVhSbEp5d25iM0JsY21GMGIzSmZiRzluYVc0bkxDZHpaWEoyWlhJbkxDZHRZV2x1ZEdWdVlXNWpaVjlwYm1sMEozMEtJQ0Fn"
    "SUhOMFlYSjBaV1E5WkdGMFlTNW5aWFFvSjJobGJIQmxjbDlwYm5adlkyRjBhVzl1Y3ljcFBUMHhJR0Z1WkNCMGVYQmxLR1Jo"
    "ZEdFdVoyVjBLQ2RvWld4d1pYSmZjR2xrSnlrcElHbHpJR2x1ZENCaGJtUWdaR0YwWVZzbmFHVnNjR1Z5WDNCcFpDZGRQakFL"
    "SUNBZ0lHbG1JSE4wWVhKMFpXUTZZV3hzYjNkbFpDNWhaR1FvSjJobGJIQmxjaWNwQ2lBZ0lDQnlZWGM5WkdGMFlTNW5aWFFv"
    "SjI5M2JtVmtYMk5vYVd4a1gyVjRhWFJ6SnlrS0lDQWdJR2xtSUdsemFXNXpkR0Z1WTJVb2NtRjNMR3hwYzNRcElHRnVaQ0Jz"
    "Wlc0b2NtRjNLVDA5YkdWdUtHRnNiRzkzWldRcElHRnVaQ0JoYkd3b2FYTnBibk4wWVc1alpTaDJMR1JwWTNRcElHRnVaQ0Iy"
    "TG1kbGRDZ25ibUZ0WlNjcElHbHVJR0ZzYkc5M1pXUWdZVzVrSUhSNWNHVW9kaTVuWlhRb0ozQnBaQ2NwS1NCcGN5QnBiblFn"
    "WVc1a0lIWmJKM0JwWkNkZFBqQWdZVzVrSUhSNWNHVW9kaTVuWlhRb0oyVjRhWFFuS1NrZ2FYTWdhVzUwSUdadmNpQjJJR2x1"
    "SUhKaGR5a2dZVzVrSUh0Mld5ZHVZVzFsSjEwZ1ptOXlJSFlnYVc0Z2NtRjNmVDA5WVd4c2IzZGxaQ0JoYm1RZ2JHVnVLSHQy"
    "V3lkd2FXUW5YU0JtYjNJZ2RpQnBiaUJ5WVhkOUtUMDliR1Z1S0dGc2JHOTNaV1FwSUdGdVpDQnVaWGgwS0haYkozQnBaQ2Rk"
    "SUdadmNpQjJJR2x1SUhKaGR5QnBaaUIyV3lkdVlXMWxKMTA5UFNkelpYSjJaWEluS1QwOVkxc25jMlZ5ZG1WeVgzQnBaQ2Rk"
    "SUdGdVpDQW9LSE4wWVhKMFpXUWdZVzVrSUc1bGVIUW9kbHNuY0dsa0oxMGdabTl5SUhZZ2FXNGdjbUYzSUdsbUlIWmJKMjVo"
    "YldVblhUMDlKMmhsYkhCbGNpY3BQVDFrWVhSaFd5ZG9aV3h3WlhKZmNHbGtKMTBnWVc1a0lDaG9aV3h3WlhKZmNHbGtJR2x6"
    "SUU1dmJtVWdiM0lnYUdWc2NHVnlYM0JwWkQwOVpHRjBZVnNuYUdWc2NHVnlYM0JwWkNkZEtTa2diM0lnS0c1dmRDQnpkR0Z5"
    "ZEdWa0lHRnVaQ0JvWld4d1pYSmZjR2xrSUdseklFNXZibVVnWVc1a0lHUmhkR0V1WjJWMEtDZG9aV3h3WlhKZmFXNTJiMk5o"
    "ZEdsdmJuTW5LU0JwY3lCT2IyNWxJR0Z1WkNCa1lYUmhMbWRsZENnbmFHVnNjR1Z5WDNCcFpDY3BJR2x6SUU1dmJtVXBLVG9L"
    "SUNBZ0lDQWdJQ0JqYUdsc1pISmxiajFiZTJzNmRsdHJYU0JtYjNJZ2F5QnBiaUFvSjI1aGJXVW5MQ2R3YVdRbkxDZGxlR2ww"
    "SnlsOUlHWnZjaUIySUdsdUlISmhkMTBLSUNBZ0lDQWdJQ0JvWld4d1pYSmZjR2xrUFdSaGRHRmJKMmhsYkhCbGNsOXdhV1Fu"
    "WFNCcFppQnpkR0Z5ZEdWa0lHVnNjMlVnVG05dVpRcHdhV1J6UFd4cGMzUW9ZMXNuYjNkdVpXUmZjR2xrY3lkZEtRcHBaaUJv"
    "Wld4d1pYSmZjR2xrSUdseklHNXZkQ0JPYjI1bElHRnVaQ0JvWld4d1pYSmZjR2xrSUc1dmRDQnBiaUJ3YVdSek9uQnBaSE11"
    "WVhCd1pXNWtLR2hsYkhCbGNsOXdhV1FwQ25BOWMzVmljSEp2WTJWemN5NXlkVzRvV3ljdlltbHVMM0J6Snl3bkxYQW5MQ2Nz"
    "Snk1cWIybHVLSE4wY2loMktTQm1iM0lnZGlCcGJpQndhV1J6S1N3bkxXOG5MQ2R3YVdROUoxMHNZMkZ3ZEhWeVpWOXZkWFJ3"
    "ZFhROVZISjFaU3gwYVcxbGIzVjBQVE1wQ25CelgydHViM2R1UFhBdWNtVjBkWEp1WTI5a1pTQnBiaUFvTUN3eEtTQmhibVFn"
    "WVd4c0tIWXVhWE5rYVdkcGRDZ3BJR1p2Y2lCMklHbHVJSEF1YzNSa2IzVjBMbk53YkdsMEtDa3BDbkJ5WlhObGJuUTljMlYw"
    "S0dsdWRDaDJLU0JtYjNJZ2RpQnBiaUJ3TG5OMFpHOTFkQzV6Y0d4cGRDZ3BLU0JwWmlCd2MxOXJibTkzYmlCbGJITmxJSE5s"
    "ZENncENuQnZjblJ6UFh0OUNtWnZjaUJ3YjNKMElHbHVJQ2c1TURBd0xETXdNREFwT2dvZ0lDQWdjRDF6ZFdKd2NtOWpaWE56"
    "TG5KMWJpaGJKeTkxYzNJdmMySnBiaTlzYzI5bUp5d25MVzVRSnl3bkxYUW5MQ2N0YVZSRFVEb25LM04wY2lod2IzSjBLU3du"
    "TFhOVVExQTZURWxUVkVWT0oxMHNZMkZ3ZEhWeVpWOXZkWFJ3ZFhROVZISjFaU3gwYVcxbGIzVjBQVE1wQ2lBZ0lDQndiM0ow"
    "YzF0emRISW9jRzl5ZENsZFBXNXZkQ0JpYjI5c0tIQXVjM1JrYjNWMExuTjBjbWx3S0NrcElHbG1JSEF1Y21WMGRYSnVZMjlr"
    "WlNCcGJpQW9NQ3d4S1NCbGJITmxJRTV2Ym1VS2NISnBiblFvYW5OdmJpNWtkVzF3Y3loN0oyTnNiMk5ySnpwN0oyMXZibTkw"
    "YjI1cFkxOXVjeWM2YzNSeUtIUnBiV1V1Ylc5dWIzUnZibWxqWDI1ektDa3BMQ2QzWVd4c1gyVndiMk5vWDI1ekp6cHpkSElv"
    "ZEdsdFpTNTBhVzFsWDI1ektDa3BmU3duYjNkdVpXUmZjR2xrY3ljNmUzTjBjaWgyS1Rvb2RpQnViM1FnYVc0Z2NISmxjMlZ1"
    "ZENCcFppQndjMTlyYm05M2JpQmxiSE5sSUU1dmJtVXBJR1p2Y2lCMklHbHVJSEJwWkhOOUxDZHdiM0owY3ljNmNHOXlkSE1z"
    "SjJ4aFlsOWhZbk5sYm5Rbk9tNXZkQ0J3WVhSb2JHbGlMbEJoZEdnb1kxc25iR0ZpSjEwcExtVjRhWE4wY3lncExDZHZkMjVs"
    "WkY5amFHbHNaRjlsZUdsMGN5YzZZMmhwYkdSeVpXNHNKMmhsYkhCbGNsOXdhV1FuT21obGJIQmxjbDl3YVdRc0ozVnVaWGh3"
    "WldOMFpXUmZabUZwYkhWeVpWOXZZbk5sY25aaGRHbHZiaWM2YjNWMFpYSmZiMkp6WlhKMllYUnBiMjRzSjNWdVpYaHdaV04w"
    "WldSZmIySnpaWEoyWVhScGIyNWZjSEp2YW1WamRHbHZiaWM2Y0hKdmFtVmpkR2x2Ym4wcEtRbz0iLAogICAgICAiYnl0ZXMi"
    "OiA0NDI0LAogICAgICAic2hhMjU2IjogImI3ZDM1MDQyNDVhZTk0YTU4NGI0NTVmYTY1ODI2ZmJhYTc3NTllYzdhNTBhN2I4"
    "MWMyY2Y4Mjg0YzlmOTA1MWYiCiAgICB9LAogICAgIk1BUktFUiI6IHsKICAgICAgImI2NCI6ICJhVzF3YjNKMElHOXpMSEJo"
    "ZEdoc2FXSXNjM1JoZEN4emVYTUtiR0ZpUFhCaGRHaHNhV0l1VUdGMGFDaHplWE11WVhKbmRsc3hYU2s3WjNWaGNtUTlhVzUw"
    "S0hONWN5NWhjbWQyV3pKZEtUdHpaWEoyWlhJOWFXNTBLSE41Y3k1aGNtZDJXek5kS1FwcFppQm5kV0Z5WkR3OU1DQnZjaUJ6"
    "WlhKMlpYSThQVEFnYjNJZ1ozVmhjbVE5UFhObGNuWmxjanB5WVdselpTQldZV3gxWlVWeWNtOXlLQ2R2ZDI1bFpGOWpiMjUw"
    "WlhoMFgybHVkbUZzYVdRbktRcHBibVp2UFd4aFlpNXNjM1JoZENncENtbG1JRzV2ZENBb2MzUmhkQzVUWDBsVFJFbFNLR2x1"
    "Wm04dWMzUmZiVzlrWlNrZ1lXNWtJSE4wWVhRdVUxOUpUVTlFUlNocGJtWnZMbk4wWDIxdlpHVXBQVDB3Ynpjd01DQmhibVFn"
    "YVc1bWJ5NXpkRjkxYVdROVBXOXpMbWRsZEhWcFpDZ3BLVHB5WVdselpTQldZV3gxWlVWeWNtOXlLQ2R2ZDI1bFpGOWpiMjUw"
    "WlhoMFgybHVkbUZzYVdRbktRcHZjeTVyYVd4c0tHZDFZWEprTERBcE8yOXpMbXRwYkd3b2MyVnlkbVZ5TERBcENtbG1JQ2hz"
    "WVdJdkozTjBiM0FuS1M1bGVHbHpkSE1vS1NCdmNpQW9iR0ZpTHlkMWFTMW1ZV2xzZFhKbEp5a3VaWGhwYzNSektDazZjbUZw"
    "YzJVZ1ZtRnNkV1ZGY25KdmNpZ25iM2R1WldSZlkyOXVkR1Y0ZEY5cGJuWmhiR2xrSnlrS1ptUTliM011YjNCbGJpaHNZV0l2"
    "SjJKeWIzZHpaWEl0Y0hKbGNHRnlaV1FuTEc5ekxrOWZWMUpQVGt4WmZHOXpMazlmUTFKRlFWUjhiM011VDE5RldFTk1mRzl6"
    "TGs5ZlRrOUdUMHhNVDFjc01HODJNREFwQ205ekxtTnNiM05sS0daa0tRcHdjbWx1ZENnbmV5SndjbVZ3WVhKbFpGOXRZWEpy"
    "WlhKZmQzSnBkSFJsYmlJNmRISjFaWDBuS1FvPSIsCiAgICAgICJieXRlcyI6IDYyNiwKICAgICAgInNoYTI1NiI6ICIxM2M2"
    "NTQ5ZTAzMjhhZDY5NTc2OTkzOGNiYzBiZTdlYjQ3ZDdhMTBlYTlmMDBjNGQ4OTc2NTU2MDM0MzdhOTQ5IgogICAgfSwKICAg"
    "ICJQRVJTSVNUIjogewogICAgICAiYjY0IjogImFXMXdiM0owSUdwemIyNHNiM01zY0dGMGFHeHBZaXh6ZVhNS2NHRjBhRDF3"
    "WVhSb2JHbGlMbEJoZEdnb2MzbHpMbUZ5WjNaYk1WMHBDbkpsWTI5eVpEMXFjMjl1TG14dllXUnpLSE41Y3k1aGNtZDJXekpk"
    "S1FvaklFTnZiblpsY25RZ1pHVmphVzFoYkNCemRISnBibWR6SUdScGNtVmpkR3g1SUhSdklGQjVkR2h2YmlCcGJuUmxaMlZ5"
    "Y3l3Z1lYWnZhV1JwYm1jZ1NsTWdUblZ0WW1WeUlISnZkVzVrYVc1bkxncGtaV1lnWTJ4dlkydHpLSFpoYkhWbEtUb0tJQ0Fn"
    "SUdsbUlHbHphVzV6ZEdGdVkyVW9kbUZzZFdVc1pHbGpkQ2s2Q2lBZ0lDQWdJQ0FnWm05eUlHc3NkaUJwYmlCc2FYTjBLSFpo"
    "YkhWbExtbDBaVzF6S0NrcE9nb2dJQ0FnSUNBZ0lDQWdJQ0JwWmlCcklHbHVJQ2duYlc5dWIzUnZibWxqWDI1ekp5d25kMkZz"
    "YkY5bGNHOWphRjl1Y3ljcElHRnVaQ0JwYzJsdWMzUmhibU5sS0hZc2MzUnlLVG9LSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJR0Z6"
    "YzJWeWRDQjJMbWx6WkdWamFXMWhiQ2dwSUdGdVpDQnNaVzRvZGlrOFBURTVJR0Z1WkNBd1BEMXBiblFvZGlrOE1pb3FOak1L"
    "SUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJSFpoYkhWbFcydGRQV2x1ZENoMktRb2dJQ0FnSUNBZ0lDQWdJQ0JsYkhObE9tTnNiMk5y"
    "Y3loMktRb2dJQ0FnWld4cFppQnBjMmx1YzNSaGJtTmxLSFpoYkhWbExHeHBjM1FwT2dvZ0lDQWdJQ0FnSUdadmNpQjJJR2x1"
    "SUhaaGJIVmxPbU5zYjJOcmN5aDJLUXBqYkc5amEzTW9jbVZqYjNKa0tRcG1aRDF2Y3k1dmNHVnVLSEJoZEdnc2IzTXVUMTlY"
    "VWs5T1RGbDhiM011VDE5RFVrVkJWSHh2Y3k1UFgwVllRMHg4YjNNdVQxOU9UMFpQVEV4UFZ5d3diell3TUNrS2QybDBhQ0J2"
    "Y3k1bVpHOXdaVzRvWm1Rc0ozY25MR1Z1WTI5a2FXNW5QU2RoYzJOcGFTY3BJR0Z6SUdZNkNpQWdJQ0JtTG5keWFYUmxLR3B6"
    "YjI0dVpIVnRjSE1vY21WamIzSmtMSE52Y25SZmEyVjVjejFVY25WbExHbHVaR1Z1ZEQweUtTc25YRzRuS1R0bUxtWnNkWE5v"
    "S0NrN2IzTXVabk41Ym1Nb1ppNW1hV3hsYm04b0tTa0tjSEpwYm5Rb2FuTnZiaTVrZFcxd2N5aDdKM2R5YVhSMFpXNWZaWGhq"
    "YkhWemFYWmxKenBVY25WbGZTa3BDZz09IiwKICAgICAgImJ5dGVzIjogODA1LAogICAgICAic2hhMjU2IjogIjgxOWE3Nzg1"
    "NjI1Y2UyZjc4YTljYjAzYjVmMjQyZGJmYTY3MjhmOWY1MDdiMjQyZTNjMDQ4ODg3YWQwMWVhMzAiCiAgICB9CiAgfSwKICAi"
    "Y2FzZV9wbGFuIjogWwogICAgewogICAgICAibmFtZSI6ICJwaGFzZV9lbnRyeV90aGVuX2NvbnRpbnVhdGlvbiIsCiAgICAg"
    "ICJncm91cCI6ICJwaGFzZV9iaW5kaW5nIgogICAgfSwKICAgIHsKICAgICAgIm5hbWUiOiAicGFzc3dvcmRfc3Vydml2ZXNf"
    "dW50aWxfcGFzc3dvcmRfZGlzcGF0Y2giLAogICAgICAiZ3JvdXAiOiAic2VjcmV0X2xpZmV0aW1lIgogICAgfSwKICAgIHsK"
    "ICAgICAgIm5hbWUiOiAibWlzc2luZ19wYXNzd29yZF9yZWZ1c2VzX3dpdGhvdXRfaW5wdXQiLAogICAgICAiZ3JvdXAiOiAi"
    "c2VjcmV0X2xpZmV0aW1lIgogICAgfSwKICAgIHsKICAgICAgIm5hbWUiOiAidW5vd25lZF9jb250ZXh0X3JlZnVzZXNfd2l0"
    "aG91dF9vcGVyYXRpb25zIiwKICAgICAgImdyb3VwIjogInBoYXNlX2JpbmRpbmciCiAgICB9LAogICAgewogICAgICAibmFt"
    "ZSI6ICJ1bmRlY2xhcmVkX3JlZl9yZWZ1c2VzX2lucHV0IiwKICAgICAgImdyb3VwIjogInBoYXNlX2JpbmRpbmciCiAgICB9"
    "LAogICAgewogICAgICAibmFtZSI6ICJzdGFsZV9yZWZfY2Fubm90X2Nyb3NzX2NlbGxzIiwKICAgICAgImdyb3VwIjogInBo"
    "YXNlX2JpbmRpbmciCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJpbnZhbGlkX2tpbmRfcmVwZWF0ZWRfY2xlYW51cF9j"
    "bGVhcnNfYmVmb3JlX2F3YWl0IiwKICAgICAgImdyb3VwIjogInNlY3JldF9saWZldGltZSIKICAgIH0sCiAgICB7CiAgICAg"
    "ICJuYW1lIjogImhlbHBlcl96ZXJvX2ZpbmFsX3NuYXBzaG90X3RoZW5fY2xlYW51cCIsCiAgICAgICJncm91cCI6ICJoZWxw"
    "ZXJfaGFuZG9mZiIKICAgIH0sCiAgICB7CiAgICAgICJuYW1lIjogImhlbHBlcl96ZXJvX21pc3NpbmdfcGFnZV9zdGlsbF9m"
    "YWlscyIsCiAgICAgICJncm91cCI6ICJoZWxwZXJfaGFuZG9mZiIKICAgIH0sCiAgICB7CiAgICAgICJuYW1lIjogImhlbHBl"
    "cl9ub256ZXJvX2ZpbmFsX3N0aWxsX3JlZnVzZXMiLAogICAgICAiZ3JvdXAiOiAiaGVscGVyX2hhbmRvZmYiCiAgICB9LAog"
    "ICAgewogICAgICAibmFtZSI6ICJoZWxwZXJfemVyb19vdGhlcl9raW5kX3N0aWxsX3JlZnVzZXMiLAogICAgICAiZ3JvdXAi"
    "OiAiaGVscGVyX2hhbmRvZmYiCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJoZWxwZXJfemVyb19wcmVwYXJlZF9waGFz"
    "ZV9zdGlsbF9yZWZ1c2VzIiwKICAgICAgImdyb3VwIjogImhlbHBlcl9oYW5kb2ZmIgogICAgfSwKICAgIHsKICAgICAgIm5h"
    "bWUiOiAicGFydGlhbF9saW5lX2RyYWluc19iZWZvcmVfb3V0cHV0X2FuZF9uZXh0X2NlbGwiLAogICAgICAiZ3JvdXAiOiAi"
    "cGFydGlhbF9mcmFtaW5nIgogICAgfSwKICAgIHsKICAgICAgIm5hbWUiOiAicGFydGlhbF9leGFjdF9jYXBfaXNfYWNjZXB0"
    "ZWQiLAogICAgICAiZ3JvdXAiOiAicGFydGlhbF9mcmFtaW5nIgogICAgfSwKICAgIHsKICAgICAgIm5hbWUiOiAicGFydGlh"
    "bF9vdmVyX2NhcF9yZWZ1c2VzIiwKICAgICAgImdyb3VwIjogInBhcnRpYWxfZnJhbWluZyIKICAgIH0sCiAgICB7CiAgICAg"
    "ICJuYW1lIjogInBhcnRpYWxfam9pbmVkX2NvbnRyb2xsZXJfcmVmdXNlcyIsCiAgICAgICJncm91cCI6ICJwYXJ0aWFsX2Zy"
    "YW1pbmciCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJwYXJ0aWFsX3VuYXZhaWxhYmxlX2NvbnRyb2xsZXJfcmVmdXNl"
    "cyIsCiAgICAgICJncm91cCI6ICJwYXJ0aWFsX2ZyYW1pbmciCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJwYXJ0aWFs"
    "X2RlYWRsaW5lX3ByZXZlbnRzX2V4dHJhX3BvbGwiLAogICAgICAiZ3JvdXAiOiAicGFydGlhbF9mcmFtaW5nIgogICAgfSwK"
    "ICAgIHsKICAgICAgIm5hbWUiOiAicGFydGlhbF9jb21wbGV0ZWRfbGF0ZV9zdGlsbF9yZWZ1c2VzIiwKICAgICAgImdyb3Vw"
    "IjogInBhcnRpYWxfZnJhbWluZyIKICAgIH0sCiAgICB7CiAgICAgICJuYW1lIjogInJldGFpbmVkX3N0YXJ0X2J1ZGdldF9y"
    "ZWZ1c2VzX25leHRfY2VsbCIsCiAgICAgICJncm91cCI6ICJwaGFzZV9iaW5kaW5nIgogICAgfSwKICAgIHsKICAgICAgIm5h"
    "bWUiOiAiaW5jbHVzaXZlX2NsZWFudXBfZGVhZGxpbmVfZG9lc19ub3RfaW52ZW50X2pvaW4iLAogICAgICAiZ3JvdXAiOiAi"
    "Y2xlYW51cF9sYXRjaCIKICAgIH0sCiAgICB7CiAgICAgICJuYW1lIjogImZpcnN0X3BhZ2VfZmFpbHVyZV9zdXJ2aXZlc19s"
    "YXRlcl9oZWxwZXJfZXJyb3IiLAogICAgICAiZ3JvdXAiOiAiY2xlYW51cF9sYXRjaCIKICAgIH0sCiAgICB7CiAgICAgICJu"
    "YW1lIjogIm1pc3NpbmdfYWJzZW5jZV9wcm9vZl9wcmV2ZW50c19yZWxlYXNlIiwKICAgICAgImdyb3VwIjogImNsZWFudXBf"
    "bGF0Y2giCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJjbGVhbnVwX2V4Y2VwdGlvbl9pc19wcml2YXRlIiwKICAgICAg"
    "Imdyb3VwIjogImNsZWFudXBfbGF0Y2giCiAgICB9LAogICAgewogICAgICAibmFtZSI6ICJwZW5kaW5nX21ldGFkYXRhX2Nv"
    "bW1hbmRfcHJldmVudHNfcmVsZWFzZSIsCiAgICAgICJncm91cCI6ICJjbGVhbnVwX2xhdGNoIgogICAgfQogIF0sCiAgImNo"
    "ZWNrX25hbWVzIjogWwogICAgInJlYWxfZGVhZGxpbmUiLAogICAgInNvdXJjZV9lbmNvZGluZyIsCiAgICAicHJpdmFjeV9z"
    "aW5rIiwKICAgICJkaWZmX2ZyYW1pbmciLAogICAgImRpZmZfaGVhZGVycyIsCiAgICAiZGlmZl9odW5rIiwKICAgICJkaWZm"
    "X3Bvc2l0aW9uIiwKICAgICJkaWZmX2xpbmUiLAogICAgImRpZmZfZXhhY3RfbGluZSIsCiAgICAiZGlmZl9odW5rX2NvdW50"
    "IiwKICAgICJjYW5kaWRhdGVfaWRlbnRpdHkiLAogICAgImJhc2VsaW5lX2lkZW50aXR5IiwKICAgICJkaWZmX2lkZW50aXR5"
    "IiwKICAgICJlZGl0X3VuaXF1ZSIsCiAgICAiYmFzZWxpbmVfaW52ZXJzZSIsCiAgICAiY29sbGVjdG9yX2lkZW50aXR5IiwK"
    "ICAgICJjb2xsZWN0b3JfaW5fY2FuZGlkYXRlIiwKICAgICJjb2xsZWN0b3Jfc2V0IiwKICAgICJ0cmFjZV9jYXAiLAogICAg"
    "ImNhbGxfY2FwIiwKICAgICJzdG9yZV9rZXkiLAogICAgInBhc3N3b3JkX3N0b3JlX3ZhbHVlIiwKICAgICJyZWNlaXB0X3By"
    "ZWNlZGVzX2NvbXBhcmlzb24iLAogICAgInJlYWR5X3JlY2VpcHRfb3JkZXIiLAogICAgImxvYWRfa2V5IiwKICAgICJjb2xs"
    "ZWN0b3JfY29tbWFuZCIsCiAgICAiY29sbGVjdG9yX3F1b3RpbmciLAogICAgImNvbGxlY3Rvcl9hbGxvd2xpc3QiLAogICAg"
    "ImNvbGxlY3Rvcl9hcmd1bWVudHMiLAogICAgIm1hcmtlcl9ib3VuZCIsCiAgICAiY2xlYXJfYmVmb3JlX2NsZWFudXBfYXdh"
    "aXQiLAogICAgImxhdGNoX2JlZm9yZV9jbGVhbnVwIiwKICAgICJjbGVhcl9zdXJ2aXZlc19jbGVhbnVwX2F3YWl0IiwKICAg"
    "ICJyZWFkYmFja19ib3VuZCIsCiAgICAicGVyc2lzdF9ib3VuZCIsCiAgICAiYm91bmRfYnJvd3Nlcl9oYW5kbGVzIiwKICAg"
    "ICJib3VuZF9yZWZlcmVuY2UiLAogICAgImJvdW5kX2NvbnRyb2xsZXIiLAogICAgIm5hdmlnYXRpb25fYWxsb3dsaXN0IiwK"
    "ICAgICJzbmFwc2hvdF9wdWJsaWNfb25seSIsCiAgICAiY2xpY2tfcm91dGUiLAogICAgInR5cGVfcmVwbGFjZSIsCiAgICAi"
    "dHlwZV92YWx1ZSIsCiAgICAia2lsbF9ib3VuZF9vd25lZF9waWQiLAogICAgImVuZF9ib3VuZF9zZXNzaW9uIiwKICAgICJ3"
    "aW5kb3dzX2JvdW5kX3BpZCIsCiAgICAiY2VsbF9jYXAiLAogICAgImNlbGxfb3V0cHV0IiwKICAgICJub193aG9sZTYwX2Ny"
    "ZWRpdCIsCiAgICAibm9fam91cm5leV9jcmVkaXQiLAogICAgIm9ic2VydmVkX29ubHkiLAogICAgImZpcnN0X2ZhaWx1cmVf"
    "bWF0Y2hlcyIsCiAgICAicmVsZWFzZV9zZXBhcmF0ZSIsCiAgICAic2luZ2xlX2NsZWFudXBfcHJvdG9jb2wiLAogICAgInBh"
    "c3N3b3JkX2NsZWFyZWQiLAogICAgImZpeHR1cmVfbGluZV9zaXplIiwKICAgICJlbnRyeV9waGFzZV9hbmRfaGVscGVyIiwK"
    "ICAgICJyZWFkeV9yZWNlaXB0X3Byb3ZlZCIsCiAgICAic3RhcnR1cF9idWRnZXRfcmV0YWluZWQiLAogICAgInNlY3JldF9z"
    "dXJ2aXZlc19ub25wYXNzd29yZCIsCiAgICAiaW5wdXRfcm91dGVfc2VxdWVuY2UiLAogICAgInBhc3N3b3JkX2Rpc3BhdGNo"
    "X2NvbnN1bWVzX29uY2UiLAogICAgIm5vX2lucHV0X29uX3JlZnVzYWwiLAogICAgInVub3duZWRfcmVmdXNhbCIsCiAgICAi"
    "ZnJlc2hfcmVmX3JlcGxhY2VzX2NvbnN1bWVkIiwKICAgICJzaW5nbGVfdXNlX3JlZiIsCiAgICAicmVwZWF0X2NsZWFyX25v"
    "X3JlcGVhdF9hd2FpdCIsCiAgICAicmVhZF9vbmx5X2hhbmRvZmYiLAogICAgInplcm9fb25seV9kZWNsYXJlZF9maW5hbCIs"
    "CiAgICAib25lX2ZpbmFsX3NuYXBzaG90IiwKICAgICJmaW5hbF9wYWdlX3Byb3ZlZCIsCiAgICAicHJlcGFyZWRfcGhhc2Vf"
    "cmVmdXNhbCIsCiAgICAicGFydGlhbF9ldmVudF9wcm9jZXNzZWRfYmVmb3JlX291dHB1dCIsCiAgICAibm9fcGFydGlhbF9z"
    "dG9yZSIsCiAgICAibm9fcGFydGlhbF9jYXJyeV9uZXh0X2NlbGwiLAogICAgIm5vX2V4dHJhX2FjdGl2ZV9wb2xsX2F0X2Rl"
    "YWRsaW5lIiwKICAgICJub19zbmFwc2hvdF9hZnRlcl9kZWFkbGluZSIsCiAgICAibm9fam9pbl9jcmVkaXRfYXRfaW5jbHVz"
    "aXZlX2xpbWl0IiwKICAgICJkcml2ZXJfZmFpbHVyZV9yZXRhaW5lZCIsCiAgICAidW5pcXVlX2Nhc2VfbmFtZXMiLAogICAg"
    "InVuZXhwZWN0ZWQiLAogICAgIm1ldGFkYXRhX2NhcCIKICBdCn07Cgpjb25zdCBMSU1JVF9NUz0zMDAwMCwgTUFYX0NFTExT"
    "PTc1LCBNQVhfQ0FMTFM9MjUwMCwgTUFYX1RSQUNFPTUxMjsKY29uc3QgUEFTU1dPUkQ9IkQwMV9TWU5USEVUSUNfUEFTU1dP"
    "UkRfU0VOVElORUxfMDEyMzQ1Njc4OUFCQ0RFRkciOwpjb25zdCBQQVJUSUFMPSJEMDFfU1lOVEhFVElDX1BBUlRJQUxfU0VO"
    "VElORUxfUFJJVkFURSI7CmNvbnN0IEVYQ0VQVElPTj0iRDAxX1NZTlRIRVRJQ19FWENFUFRJT05fU0VOVElORUxfUFJJVkFU"
    "RSI7CmNvbnN0IFBSSVZBVEU9W1BBU1NXT1JELFBBUlRJQUwsRVhDRVBUSU9OXTsKY29uc3QgT0JTPSJkMDFfaW1tZWRpYXRl"
    "X29ic2VydmF0aW9uX3JlY29yZCIsIE9XTj0iZDAxX2ltbWVkaWF0ZV9vd25lZF9oYW5kbGVzIjsKY29uc3QgUFc9ImQwMV9m"
    "cmVzaF9wYXNzd29yZF9pbnB1dCIsIEVYSVQ9T2JqZWN0LmZyZWV6ZSh7fSk7CmNvbnN0IEZBVUxUUz1uZXcgV2Vha01hcCgp"
    "Owpjb25zdCByZXN1bHQ9e3NjaGVtYToicmlhdXRoLmQwMS1jb250aW51YXRpb24tbWVtb3J5L3YxIiwKICBzb3VyY2U6bnVs"
    "bCxwbGFubmVkOkJJTkRJTkcuY2FzZV9wbGFuLGF0dGVtcHRlZDpbXSxjb21wbGV0ZWQ6W10sCiAgY29tcGxldGVkX2dyb3Vw"
    "czp7fSxjZWxsczowLHN0dWJfY291bnRzOnt9LGFzc2VydGlvbnNfY29tcGxldGVkOjAsCiAgcHJpdmFjeV9jaGVja3NfY29t"
    "cGxldGVkOjAsZmlyc3RfZmFpbHVyZTpudWxsLHVucmVhY2hlZDpbXSxlbGFwc2VkX21zOm51bGwsCiAgZnVsbF9jYW5kaWRh"
    "dGVfb25seTp0cnVlLGFjdHVhbF90b29sc19vcl9wcm9kdWN0OmZhbHNlfTsKZnVuY3Rpb24gZmFpbHVyZShjb2RlKXtjb25z"
    "dCBlPU9iamVjdC5mcmVlemUoe30pO0ZBVUxUUy5zZXQoZSxjb2RlKTtyZXR1cm4gZTt9CmZ1bmN0aW9uIGVuc3VyZShvayxj"
    "b2RlKXtpZighb2spdGhyb3cgZmFpbHVyZShjb2RlKTtyZXN1bHQuYXNzZXJ0aW9uc19jb21wbGV0ZWQrKzt9CmZ1bmN0aW9u"
    "IGNsb2NrR3VhcmQoKXtlbnN1cmUocGVyZm9ybWFuY2Uubm93KCk8TElNSVRfTVMsInJlYWxfZGVhZGxpbmUiKTt9CmNvbnN0"
    "IGNsb25lPXY9PnY9PT11bmRlZmluZWQ/dW5kZWZpbmVkOkpTT04ucGFyc2UoSlNPTi5zdHJpbmdpZnkodikpOwpjb25zdCBz"
    "aGE9cz0+Y3JlYXRlSGFzaCgic2hhMjU2IikudXBkYXRlKHMpLmRpZ2VzdCgiaGV4Iik7CmZ1bmN0aW9uIGRlY29kZShlbmNv"
    "ZGVkKXtlbnN1cmUodHlwZW9mIGVuY29kZWQ9PT0ic3RyaW5nIiYmCiAgL14oPzpbQS1aYS16MC05Ky9dezR9KSooPzpbQS1a"
    "YS16MC05Ky9dezJ9PT18W0EtWmEtejAtOSsvXXszfT0pPyQvLnRlc3QoZW5jb2RlZCksCiAgInNvdXJjZV9lbmNvZGluZyIp"
    "O2NvbnN0IGI9QnVmZmVyLmZyb20oZW5jb2RlZCwiYmFzZTY0Iik7CiAgZW5zdXJlKGIudG9TdHJpbmcoImJhc2U2NCIpPT09"
    "ZW5jb2RlZCwic291cmNlX2VuY29kaW5nIik7cmV0dXJuIGIudG9TdHJpbmcoInV0ZjgiKTt9CmZ1bmN0aW9uIHByaXZhY3ko"
    "dmFsdWUpewogIGNvbnN0IHM9SlNPTi5zdHJpbmdpZnkodmFsdWUpO2Vuc3VyZSh0eXBlb2Ygcz09PSJzdHJpbmciJiYhUFJJ"
    "VkFURS5zb21lKHg9PnMuaW5jbHVkZXMoeCkpLAogICAgInByaXZhY3lfc2luayIpO3Jlc3VsdC5wcml2YWN5X2NoZWNrc19j"
    "b21wbGV0ZWQrKzsKfQpmdW5jdGlvbiBpbnZlcnNlRGlmZihjYW5kaWRhdGUsZGlmZil7CiAgY29uc3QgbGluZXM9Y2FuZGlk"
    "YXRlLm1hdGNoKC9bXlxuXSpcbi9nKXx8W10sIGRzPWRpZmYubWF0Y2goL1teXG5dKlxuL2cpfHxbXTsKICBlbnN1cmUobGlu"
    "ZXMuam9pbigiIik9PT1jYW5kaWRhdGUmJmRzLmpvaW4oIiIpPT09ZGlmZiwiZGlmZl9mcmFtaW5nIik7CiAgbGV0IGN1cnNv"
    "cj0wLGk9MixvdXQ9W10saHVua3M9MDsKICBlbnN1cmUoZHNbMF09PT0iLS0tIGltbXV0YWJsZS1jNWQ0ZDE3My1jZWxsLWQ3"
    "Mjc4NzgwXG4iJiYKICAgIGRzWzFdPT09IisrKyBERVNJR04tb25seS1GMi1GMy1GNFxuIiwiZGlmZl9oZWFkZXJzIik7CiAg"
    "d2hpbGUoaTxkcy5sZW5ndGgpewogICAgY29uc3QgbT0vXkBAIC0oXGQrKSg/OiwoXGQrKSk/IFwrKFxkKykoPzosKFxkKykp"
    "PyBAQFxuJC8uZXhlYyhkc1tpKytdKTsKICAgIGVuc3VyZShtIT09bnVsbCwiZGlmZl9odW5rIik7aHVua3MrKztjb25zdCBz"
    "dGFydD1OdW1iZXIobVszXSktMTsKICAgIGVuc3VyZShjdXJzb3I8PXN0YXJ0LCJkaWZmX3Bvc2l0aW9uIik7b3V0LnB1c2go"
    "Li4ubGluZXMuc2xpY2UoY3Vyc29yLHN0YXJ0KSk7Y3Vyc29yPXN0YXJ0OwogICAgd2hpbGUoaTxkcy5sZW5ndGgmJiFkc1tp"
    "XS5zdGFydHNXaXRoKCJAQCAiKSl7CiAgICAgIGNvbnN0IGtpbmQ9ZHNbaV1bMF0sdGV4dD1kc1tpKytdLnNsaWNlKDEpOwog"
    "ICAgICBlbnN1cmUoWyIgIiwiKyIsIi0iXS5pbmNsdWRlcyhraW5kKSwiZGlmZl9saW5lIik7CiAgICAgIGlmKGtpbmQ9PT0i"
    "ICJ8fGtpbmQ9PT0iKyIpe2Vuc3VyZShsaW5lc1tjdXJzb3JdPT09dGV4dCwiZGlmZl9leGFjdF9saW5lIik7Y3Vyc29yKys7"
    "fQogICAgICBpZihraW5kPT09IiAifHxraW5kPT09Ii0iKW91dC5wdXNoKHRleHQpOwogICAgfQogIH0KICBlbnN1cmUoaHVu"
    "a3M9PT00LCJkaWZmX2h1bmtfY291bnQiKTtvdXQucHVzaCguLi5saW5lcy5zbGljZShjdXJzb3IpKTtyZXR1cm4gb3V0Lmpv"
    "aW4oIiIpOwp9CmxldCBDQU5ESURBVEUsQkFTRUxJTkUsU0NSSVBULENPTExFQ1RPUlM7CmZ1bmN0aW9uIGJpbmRTb3VyY2Uo"
    "KXsKICBDQU5ESURBVEU9ZGVjb2RlKEJJTkRJTkcuY2FuZGlkYXRlX2I2NCk7QkFTRUxJTkU9ZGVjb2RlKEJJTkRJTkcuYmFz"
    "ZWxpbmVfYjY0KTsKICBjb25zdCBkaWZmPWRlY29kZShCSU5ESU5HLmRpZmZfYjY0KTsKICBlbnN1cmUoQnVmZmVyLmJ5dGVM"
    "ZW5ndGgoQ0FORElEQVRFKT09PTMyNTAyJiZzaGEoQ0FORElEQVRFKT09PUJJTkRJTkcuY2FuZGlkYXRlX3NoYTI1NiwKICAg"
    "ICJjYW5kaWRhdGVfaWRlbnRpdHkiKTsKICBlbnN1cmUoQnVmZmVyLmJ5dGVMZW5ndGgoQkFTRUxJTkUpPT09MzE1ODEmJnNo"
    "YShCQVNFTElORSk9PT1CSU5ESU5HLmJhc2VsaW5lX3NoYTI1NiwKICAgICJiYXNlbGluZV9pZGVudGl0eSIpOwogIGVuc3Vy"
    "ZShCdWZmZXIuYnl0ZUxlbmd0aChkaWZmKT09PTIyNDMmJnNoYShkaWZmKT09PUJJTkRJTkcuZGlmZl9zaGEyNTYsImRpZmZf"
    "aWRlbnRpdHkiKTsKICBsZXQgcmVzdG9yZWQ9Q0FORElEQVRFOwogIGZvcihjb25zdCBlIG9mIFsuLi5CSU5ESU5HLmVkaXRz"
    "XS5yZXZlcnNlKCkpewogICAgZW5zdXJlKHJlc3RvcmVkLnNwbGl0KGUubmV4dCkubGVuZ3RoPT09MiwiZWRpdF91bmlxdWUi"
    "KTsKICAgIHJlc3RvcmVkPXJlc3RvcmVkLnJlcGxhY2UoZS5uZXh0LGUub2xkKTsKICB9CiAgZW5zdXJlKHJlc3RvcmVkPT09"
    "QkFTRUxJTkUmJmludmVyc2VEaWZmKENBTkRJREFURSxkaWZmKT09PUJBU0VMSU5FLCJiYXNlbGluZV9pbnZlcnNlIik7CiAg"
    "Q09MTEVDVE9SUz1PYmplY3QuZnJvbUVudHJpZXMoT2JqZWN0LmVudHJpZXMoQklORElORy5jb2xsZWN0b3JzKS5tYXAoKFtu"
    "YW1lLHZdKT0+ewogICAgY29uc3Qgc291cmNlPWRlY29kZSh2LmI2NCk7CiAgICBlbnN1cmUoQnVmZmVyLmJ5dGVMZW5ndGgo"
    "c291cmNlKT09PXYuYnl0ZXMmJnNoYShzb3VyY2UpPT09di5zaGEyNTYsImNvbGxlY3Rvcl9pZGVudGl0eSIpOwogICAgY29u"
    "c3QgcHJlZml4PSJjb25zdCAiK25hbWUrIj0iOwogICAgY29uc3QgbGluZT1DQU5ESURBVEUuc3BsaXQoIlxuIikuZmluZCh4"
    "PT54LnN0YXJ0c1dpdGgocHJlZml4KSk7CiAgICBlbnN1cmUodHlwZW9mIGxpbmU9PT0ic3RyaW5nIiYmbGluZS5lbmRzV2l0"
    "aCgiOyIpJiYKICAgICAgSlNPTi5wYXJzZShsaW5lLnNsaWNlKHByZWZpeC5sZW5ndGgsLTEpKT09PXNvdXJjZSwiY29sbGVj"
    "dG9yX2luX2NhbmRpZGF0ZSIpOwogICAgcmV0dXJuIFtuYW1lLHNvdXJjZV07CiAgfSkpOwogIGVuc3VyZShPYmplY3Qua2V5"
    "cyhDT0xMRUNUT1JTKS5zb3J0KCkuam9pbigiLCIpPT09IkNMT0NLLE1BUktFUixQRVJTSVNULFJFQURCQUNLLFNUT1AiLAog"
    "ICAgImNvbGxlY3Rvcl9zZXQiKTsKICBTQ1JJUFQ9bmV3IHZtLlNjcmlwdCgiKGFzeW5jIGZ1bmN0aW9uKCl7XG4iK0NBTkRJ"
    "REFURSsiXG59KSgpIiwKICAgIHtmaWxlbmFtZToicmV2aWV3ZWQtZDAxLW1lbW9yeS1jZWxsLmpzIixkaXNwbGF5RXJyb3Jz"
    "OmZhbHNlfSk7CiAgcmVzdWx0LnNvdXJjZT17Y2FuZGlkYXRlX3NoYTI1NjpzaGEoQ0FORElEQVRFKSxjYW5kaWRhdGVfYnl0"
    "ZXM6QnVmZmVyLmJ5dGVMZW5ndGgoQ0FORElEQVRFKSwKICAgIGJhc2VsaW5lX3NoYTI1NjpzaGEoQkFTRUxJTkUpLGJhc2Vs"
    "aW5lX2J5dGVzOkJ1ZmZlci5ieXRlTGVuZ3RoKEJBU0VMSU5FKSwKICAgIGRpZmZfc2hhMjU2OnNoYShkaWZmKSxsb2dpY19z"
    "aGEyNTY6QklORElORy5sb2dpY19zaGEyNTYsZnVsbF9pbnZlcnNlOnRydWV9Owp9CmZ1bmN0aW9uIGV2ZW50KHZhbHVlKXty"
    "ZXR1cm4gSlNPTi5zdHJpbmdpZnkodmFsdWUpKyJcbiI7fQpmdW5jdGlvbiBwdWJsaWNQYWdlKGtpbmQscmVmKXsKICBjb25z"
    "dCBub2Rlcz1raW5kPT09ImFmdGVyIj8KICAgIFt7cm9sZToiaGVhZGluZyIsbmFtZToiUHJvdGVjdGVkIGFwcGxpY2F0aW9u"
    "IGFjY2VzcyJ9LAogICAgIHtyb2xlOiJzdGF0aWN0ZXh0IixuYW1lOiJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlv"
    "biBhY2Nlc3MgaXMgYXZhaWxhYmxlLiJ9XToKICAgIGtpbmQ9PT0iYmVmb3JlIj9be3JvbGU6ImhlYWRpbmciLG5hbWU6Ikxv"
    "Y2FsIGRlbW8ifSx7cm9sZToic3RhdGljdGV4dCIsbmFtZToiU2lnbiBpbiByZXF1aXJlZC4ifV06CiAgICBraW5kPT09ImFw"
    "cGxpY2F0aW9uIj9be3JvbGU6ImhlYWRpbmciLG5hbWU6IkxvY2FsIGRlbW8ifSwKICAgICAge3JvbGU6InN0YXRpY3RleHQi"
    "LG5hbWU6IlVzZSBTaWduIGluIHRvIG9wZW4gdGhpcyBsb2NhbCBhcHBsaWNhdGlvbi4ifV06CiAgICBbe3JvbGU6InRleHRi"
    "b3giLG5hbWU6IlVzZXJuYW1lIn0se3JvbGU6InRleHRib3giLG5hbWU6IlBhc3N3b3JkIn0sCiAgICAge3JvbGU6ImJ1dHRv"
    "biIsbmFtZToiU2lnbiBpbiJ9XTsKICByZXR1cm4ge3N0cnVjdHVyZWRDb250ZW50OntzdGF0dXM6Im9rIixzbmFwc2hvdDp7"
    "Y29tcGxldGU6dHJ1ZX0sY29udGVudF9yZWZzOm5vZGVzLAogICAgcmVmczpbe3JlZixhY3Rpb25zOlsiY2xpY2siXX1dfX07"
    "Cn0KZnVuY3Rpb24gbWFrZVN0YXRlKG9wdGlvbnM9e30pewogIGNvbnN0IG93bj17cnVudGltZV9yZWxlYXNlOnRydWUsZHJp"
    "dmVyX293bmVkOnRydWUsZXhhY3RfYm91bmQ6dHJ1ZSxzaW5nbGVfY29udHJvbGxlcjp0cnVlLAogICAgcGhhc2U6ImJyb3dz"
    "ZXJfZGVjaXNpb24iLHByZXBhcmVkX2dhdGU6dHJ1ZSxmaXh0dXJlX3JlYWR5OnRydWUsbGlzdGVuZXJfcGlkX3Byb29mOnRy"
    "dWUsCiAgICBmcmVzaF9wYXRoc19wcmVmbGlnaHQ6dHJ1ZSxndWFyZF9waWQ6MTEsc2VydmVyX3BpZDoxMixicm93c2VyX3Bp"
    "ZDoxMyxoZWxwZXJfcGlkOjE0LAogICAgZXhlY19zZXNzaW9uOjE1LHN0YXJ0X2Vwb2NoX21zOjEwMDAwMDAsd29ya3NwYWNl"
    "OiIvc3ludGhldGljL3dvcmtzcGFjZSIsCiAgICBzZXNzaW9uOiJzeW50aGV0aWMtc2Vzc2lvbiIsdGFyZ2V0X2lkOiJzeW50"
    "aGV0aWMtdGFyZ2V0Iix0YWJfaWQ6InN5bnRoZXRpYy10YWIiLAogICAgbGFiOiIvc3ludGhldGljL2xhYiIsb3V0ZXJfb3V0"
    "OiIvc3ludGhldGljL291dGVyIixldmVudF9vdXQ6Ii9zeW50aGV0aWMvZXZlbnQiLAogICAgY2xlYW51cF9vdXQ6Ii9zeW50"
    "aGV0aWMvY2xlYW51cCIsZnJlc2hfcmVmczpbInAxOjEiXSxuZXh0X2RlY2lzaW9uOm51bGx9OwogIGlmKG9wdGlvbnMuZW50"
    "cnkpe293bi5waGFzZT0icHJlcGFyZWRfZW50cnkiO293bi5oZWxwZXJfcGlkPW51bGw7CiAgICBvd24uZml4dHVyZV9yZWFk"
    "eT1mYWxzZTtvd24ubGlzdGVuZXJfcGlkX3Byb29mPWZhbHNlO30KICBjb25zdCBzPXttYXA6bmV3IE1hcChbW09XTixjbG9u"
    "ZShvd24pXV0pLG5vdzpvd24uc3RhcnRfZXBvY2hfbXMscXVldWU6W10sCiAgICBvdXRwdXRzOltdLG1ldGFkYXRhOltdLHRy"
    "YWNlOltdLHNuYXBzaG90UXVldWU6W10saW5wdXRLaW5kczpbXSxjbGVhckV2ZW50czpbXSwKICAgIHN0b3BDb3VudDowLHN0"
    "b3BQcm9vZnM6W10scG9sbENvdW50OjAsbGFzdFBvbGxSZXR1cm46MCxsYXN0UmVjZWlwdDowLAogICAgcmVjZWlwdFJlYWR5"
    "UHJvb2ZzOjAscmVmQ291bnRlcjoxLGNsZWFudXA6ZmFsc2Usb3B0aW9ucyxmaXJzdEhvc3RGYXVsdDpudWxsfTsKICBpZihv"
    "cHRpb25zLnBhc3N3b3JkKXMubWFwLnNldChQVyxQQVNTV09SRCk7CiAgcmV0dXJuIHM7Cn0KZnVuY3Rpb24gdHJhY2Uocyxr"
    "aW5kLGRhdGE9e30pewogIGNsb2NrR3VhcmQoKTtlbnN1cmUocy50cmFjZS5sZW5ndGg8TUFYX1RSQUNFLCJ0cmFjZV9jYXAi"
    "KTsKICByZXN1bHQuc3R1Yl9jb3VudHNba2luZF09KHJlc3VsdC5zdHViX2NvdW50c1traW5kXXx8MCkrMTsKICBlbnN1cmUo"
    "T2JqZWN0LnZhbHVlcyhyZXN1bHQuc3R1Yl9jb3VudHMpLnJlZHVjZSgoYSxiKT0+YStiLDApPD1NQVhfQ0FMTFMsImNhbGxf"
    "Y2FwIik7CiAgY29uc3QgZW50cnk9e2tpbmQsLi4uZGF0YX07cHJpdmFjeShlbnRyeSk7cy50cmFjZS5wdXNoKGVudHJ5KTty"
    "ZXR1cm4gcy50cmFjZS5sZW5ndGg7Cn0KZnVuY3Rpb24gc3RvcmUocyxrZXksdmFsdWUpewogIGVuc3VyZShbT1dOLE9CUyxQ"
    "V10uaW5jbHVkZXMoa2V5KSwic3RvcmVfa2V5Iik7CiAgaWYoa2V5IT09UFcpcHJpdmFjeSh2YWx1ZSk7CiAgaWYoa2V5PT09"
    "UFcpe2Vuc3VyZSh2YWx1ZT09PW51bGx8fHZhbHVlPT09UEFTU1dPUkQsInBhc3N3b3JkX3N0b3JlX3ZhbHVlIik7CiAgICBp"
    "Zih2YWx1ZT09PW51bGwpcy5jbGVhckV2ZW50cy5wdXNoKHMudHJhY2UubGVuZ3RoKTt9CiAgaWYoa2V5PT09T0JTKXsKICAg"
    "IGNvbnN0IG9sZD1zLm1hcC5nZXQoT0JTKSxuPXZhbHVlLmNvbnRyb2xsZXJfb2JzZXJ2YXRpb25zLmxlbmd0aDsKICAgIGlm"
    "KG4+KG9sZD8uY29udHJvbGxlcl9vYnNlcnZhdGlvbnMubGVuZ3RoPz8wKSl7CiAgICAgIGVuc3VyZShzLnRyYWNlLmxlbmd0"
    "aDxNQVhfVFJBQ0UsInRyYWNlX2NhcCIpOwogICAgICBzLmxhc3RSZWNlaXB0PXMudHJhY2UubGVuZ3RoKzE7CiAgICAgIHMu"
    "dHJhY2UucHVzaCh7a2luZDoicmV0YWluZWRfY29udHJvbGxlcl9yZWNlaXB0In0pOwogICAgICBlbnN1cmUocy5sYXN0UmVj"
    "ZWlwdD5zLmxhc3RQb2xsUmV0dXJuLCJyZWNlaXB0X3ByZWNlZGVzX2NvbXBhcmlzb24iKTsKICAgIH0KICB9CiAgaWYoa2V5"
    "PT09T1dOJiZ2YWx1ZS5maXh0dXJlX3JlYWR5PT09dHJ1ZSYmcy5tYXAuZ2V0KE9XTik/LmZpeHR1cmVfcmVhZHkhPT10cnVl"
    "KXsKICAgIGVuc3VyZShzLmxhc3RSZWNlaXB0PnMubGFzdFBvbGxSZXR1cm4mJnMubGFzdFJlY2VpcHQ+MCwicmVhZHlfcmVj"
    "ZWlwdF9vcmRlciIpOwogICAgcy5yZWNlaXB0UmVhZHlQcm9vZnMrKzsKICB9CiAgcy5tYXAuc2V0KGtleSxjbG9uZSh2YWx1"
    "ZSkpOwp9CmZ1bmN0aW9uIGxvYWQocyxrZXkpe2Vuc3VyZShbT1dOLE9CUyxQV10uaW5jbHVkZXMoa2V5KSwibG9hZF9rZXki"
    "KTtyZXR1cm4gY2xvbmUocy5tYXAuZ2V0KGtleSkpO30KZnVuY3Rpb24gZGVjaXNpb24ocyxraW5kLHJlZil7CiAgY29uc3Qg"
    "bz1sb2FkKHMsT1dOKTtvLm5leHRfZGVjaXNpb249e2tpbmQsZXhwZWN0ZWRfcm9sZToidGV4dGJveCIsZXhwZWN0ZWRfbGFi"
    "ZWw6IlVzZXJuYW1lIn07CiAgaWYocmVmIT09dW5kZWZpbmVkKW8ubmV4dF9kZWNpc2lvbi5yZWY9cmVmOwogIHMubWFwLnNl"
    "dChPV04sbyk7Cn0KZnVuY3Rpb24gcXVvdGVkQXJncyhjbWQpewogIGVuc3VyZSh0eXBlb2YgY21kPT09InN0cmluZyImJmNt"
    "ZC5zdGFydHNXaXRoKCJweXRob24zIC1jICIpLCJjb2xsZWN0b3JfY29tbWFuZCIpOwogIGxldCBpPTExLG91dD1bXTsKICB3"
    "aGlsZShpPGNtZC5sZW5ndGgpewogICAgZW5zdXJlKGNtZFtpXT09PSInIiwiY29sbGVjdG9yX3F1b3RpbmciKTtpKys7bGV0"
    "IHRleHQ9IiI7CiAgICBmb3IoOzspewogICAgICBlbnN1cmUoaTxjbWQubGVuZ3RoLCJjb2xsZWN0b3JfcXVvdGluZyIpOwog"
    "ICAgICBpZihjbWQuc3RhcnRzV2l0aCgiJ1xcJyciLGkpKXt0ZXh0Kz0iJyI7aSs9NDtjb250aW51ZTt9CiAgICAgIGlmKGNt"
    "ZFtpXT09PSInIil7aSsrO2JyZWFrO310ZXh0Kz1jbWRbaSsrXTsKICAgIH0KICAgIG91dC5wdXNoKHRleHQpOwogICAgaWYo"
    "aT09PWNtZC5sZW5ndGgpYnJlYWs7CiAgICBlbnN1cmUoY21kW2ldPT09IiAiLCJjb2xsZWN0b3JfcXVvdGluZyIpO2krKzsK"
    "ICB9CiAgcmV0dXJuIG91dDsKfQphc3luYyBmdW5jdGlvbiBsb2NhbChzLGFyZ3MpewogIGNvbnN0IGFyZ3Y9cXVvdGVkQXJn"
    "cyhhcmdzLmNtZCksc291cmNlPWFyZ3Yuc2hpZnQoKTsKICBjb25zdCBsYWJlbD1PYmplY3Qua2V5cyhDT0xMRUNUT1JTKS5m"
    "aW5kKGs9PkNPTExFQ1RPUlNba109PT1zb3VyY2UpOwogIGVuc3VyZShsYWJlbCE9PXVuZGVmaW5lZCYmYXJncy53b3JrZGly"
    "PT09Ii9zeW50aGV0aWMvd29ya3NwYWNlIiwiY29sbGVjdG9yX2FsbG93bGlzdCIpOwogIHRyYWNlKHMsImNvbGxlY3Rvcl8i"
    "K2xhYmVsLnRvTG93ZXJDYXNlKCkpOwogIGNvbnN0IG89bG9hZChzLE9XTikscmVjPWxvYWQocyxPQlMpLGNsb2NrPXsKICAg"
    "IG1vbm90b25pY19uczpTdHJpbmcoQmlnSW50KHMubm93KSoxMDAwMDAwbiksd2FsbF9lcG9jaF9uczpTdHJpbmcoQmlnSW50"
    "KHMubm93KSoxMDAwMDAwbil9OwogIGxldCBkYXRhOwogIGlmKGxhYmVsPT09IkNMT0NLIil7ZW5zdXJlKGFyZ3YubGVuZ3Ro"
    "PT09MCwiY29sbGVjdG9yX2FyZ3VtZW50cyIpO2RhdGE9Y2xvY2s7fQogIGVsc2UgaWYobGFiZWw9PT0iTUFSS0VSIil7CiAg"
    "ICBlbnN1cmUoYXJndi5sZW5ndGg9PT0zJiZhcmd2WzBdPT09by5sYWImJmFyZ3ZbMV09PT0iMTEiJiZhcmd2WzJdPT09IjEy"
    "IiwibWFya2VyX2JvdW5kIik7CiAgICBkYXRhPXtwcmVwYXJlZF9tYXJrZXJfd3JpdHRlbjp0cnVlfTsKICB9IGVsc2UgaWYo"
    "bGFiZWw9PT0iU1RPUCIpewogICAgZW5zdXJlKGFyZ3YubGVuZ3RoPT09MSwiY29sbGVjdG9yX2FyZ3VtZW50cyIpO2NvbnN0"
    "IGM9SlNPTi5wYXJzZShhcmd2WzBdKTtwcml2YWN5KGMpOwogICAgZW5zdXJlKGxvYWQocyxQVyk9PT1udWxsLCJjbGVhcl9i"
    "ZWZvcmVfY2xlYW51cF9hd2FpdCIpOwogICAgZW5zdXJlKGMuZmlyc3RfZmFpbHVyZT09PXJlYy5maXJzdF9mYWlsdXJlJiYK"
    "ICAgICAgYy5maXJzdF9vYnNlcnZhdGlvbl93YWxsX21zPT09cmVjLmZpcnN0X29ic2VydmF0aW9uX3dhbGxfbXMsImxhdGNo"
    "X2JlZm9yZV9jbGVhbnVwIik7CiAgICBzLnN0b3BQcm9vZnMucHVzaCh0cnVlKTtzLnN0b3BDb3VudCsrO3MuY2xlYW51cD10"
    "cnVlOwogICAgYXdhaXQgUHJvbWlzZS5yZXNvbHZlKCk7ZW5zdXJlKGxvYWQocyxQVyk9PT1udWxsLCJjbGVhcl9zdXJ2aXZl"
    "c19jbGVhbnVwX2F3YWl0Iik7CiAgICBkYXRhPXtjbG9jayxldmVudF9zdGF0ZToid3JpdHRlbiIsbWFya2VyX3N0YXRlOiJz"
    "dG9wX3JlcXVlc3RlZCJ9OwogIH0gZWxzZSBpZihsYWJlbD09PSJSRUFEQkFDSyIpewogICAgZW5zdXJlKGFyZ3YubGVuZ3Ro"
    "PT09MSwiY29sbGVjdG9yX2FyZ3VtZW50cyIpO2NvbnN0IGM9SlNPTi5wYXJzZShhcmd2WzBdKTtwcml2YWN5KGMpOwogICAg"
    "ZW5zdXJlKGMubGFiPT09by5sYWImJmMuc2VydmVyX3BpZD09PTEyJiZjLmhlbHBlcl9waWQ9PT1vLmhlbHBlcl9waWQsInJl"
    "YWRiYWNrX2JvdW5kIik7CiAgICBjb25zdCBuYW1lcz1bIndob2FtaSIsImRpc2NvdmVyeSIsImNvbmZpZGVudGlhbF9jbGll"
    "bnRfY3JlYXRlIiwib3BlcmF0b3JfbG9naW4iLCJzZXJ2ZXIiLCJtYWludGVuYW5jZV9pbml0Il07CiAgICBjb25zdCBwaWRz"
    "PVsyMSwyMiwyMywyNCwxMiwyNV07aWYoby5oZWxwZXJfcGlkIT09bnVsbCl7bmFtZXMucHVzaCgiaGVscGVyIik7cGlkcy5w"
    "dXNoKDE0KTt9CiAgICBkYXRhPXtjbG9jayxvd25lZF9waWRzOk9iamVjdC5mcm9tRW50cmllcyhjLm93bmVkX3BpZHMubWFw"
    "KHA9PltTdHJpbmcocCksdHJ1ZV0pKSwKICAgICAgcG9ydHM6eyI5MDAwIjp0cnVlLCIzMDAwIjp0cnVlfSxsYWJfYWJzZW50"
    "OiFzLm9wdGlvbnMuYWJzZW5jZV9taXNzaW5nLAogICAgICBvd25lZF9jaGlsZF9leGl0czpuYW1lcy5tYXAoKG5hbWUsaSk9"
    "Pih7bmFtZSxwaWQ6cGlkc1tpXSxleGl0OjB9KSksCiAgICAgIGhlbHBlcl9waWQ6by5oZWxwZXJfcGlkLHVuZXhwZWN0ZWRf"
    "ZmFpbHVyZV9vYnNlcnZhdGlvbjp7b2JzZXJ2ZWQ6ZmFsc2UsZGlhZ25vc3RpYzpudWxsfSwKICAgICAgdW5leHBlY3RlZF9v"
    "YnNlcnZhdGlvbl9wcm9qZWN0aW9uOiJ2YWxpZCJ9OwogIH0gZWxzZSB7CiAgICBlbnN1cmUoYXJndi5sZW5ndGg9PT0yJiZh"
    "cmd2WzBdPT09by5jbGVhbnVwX291dCwicGVyc2lzdF9ib3VuZCIpOwogICAgY29uc3QgcmVjb3JkPUpTT04ucGFyc2UoYXJn"
    "dlsxXSk7cHJpdmFjeShyZWNvcmQpOwogICAgZW5zdXJlKEJ1ZmZlci5ieXRlTGVuZ3RoKEpTT04uc3RyaW5naWZ5KHJlY29y"
    "ZCkpPD0yNjIxNDQsIm1ldGFkYXRhX2NhcCIpO3MubWV0YWRhdGEucHVzaChjbG9uZShyZWNvcmQpKTsKICAgIGlmKHMub3B0"
    "aW9ucy5tZXRhZGF0YV9wZW5kaW5nKXJldHVybiB7c2Vzc2lvbl9pZDo5OSxvdXRwdXQ6IiJ9OwogICAgZGF0YT17d3JpdHRl"
    "bl9leGNsdXNpdmU6dHJ1ZX07CiAgfQogIHJldHVybiB7ZXhpdF9jb2RlOjAsb3V0cHV0OkpTT04uc3RyaW5naWZ5KGRhdGEp"
    "fTsKfQpmdW5jdGlvbiBib3VuZChzLGFyZ3MscmVmPWZhbHNlKXsKICBjb25zdCBvPWxvYWQocyxPV04pO2Vuc3VyZShhcmdz"
    "LnNlc3Npb249PT1vLnNlc3Npb24mJmFyZ3MudGFyZ2V0X2lkPT09by50YXJnZXRfaWQmJgogICAgYXJncy50YWJfaWQ9PT1v"
    "LnRhYl9pZCwiYm91bmRfYnJvd3Nlcl9oYW5kbGVzIik7CiAgaWYocmVmKWVuc3VyZSh0eXBlb2YgYXJncy5yZWY9PT0ic3Ry"
    "aW5nIiYmL15wWzAtOV0rOlswLTldKyQvLnRlc3QoYXJncy5yZWYpLCJib3VuZF9yZWZlcmVuY2UiKTsKfQphc3luYyBmdW5j"
    "dGlvbiBwb2xsKHMsYXJncyl7CiAgZW5zdXJlKGFyZ3Muc2Vzc2lvbl9pZD09PTE1JiZhcmdzLmNoYXJzPT09IiIsImJvdW5k"
    "X2NvbnRyb2xsZXIiKTsKICB0cmFjZShzLCJjb250cm9sbGVyX3BvbGwiLHtjbGVhbnVwOnMuY2xlYW51cH0pO3MucG9sbENv"
    "dW50Kys7CiAgbGV0IHI7CiAgaWYocy5jbGVhbnVwKXsKICAgIHI9e2V4aXRfY29kZTowLG91dHB1dDpldmVudChzLm9wdGlv"
    "bnMuY2xlYW51cF9oZWxwZXJfZXJyb3I/CiAgICAgIHtoZWxwZXJfY29tcGxldGVkOnRydWUsZXhpdDoxLGZpeHR1cmVfZmlu"
    "aXNoZWQ6dHJ1ZX06e2ZpeHR1cmVfZmluaXNoZWQ6dHJ1ZX0pfTsKICB9IGVsc2UgewogICAgY29uc3QgbmV4dD1zLnF1ZXVl"
    "LnNoaWZ0KCk/P3t9OwogICAgaWYobmV4dC50aHJvdyl7dGhyb3cgbmV3IEVycm9yKEVYQ0VQVElPTik7fQogICAgaWYobmV4"
    "dC5hdCE9PXVuZGVmaW5lZClzLm5vdz0xMDAwMDAwK25leHQuYXQ7CiAgICByPXtvdXRwdXQ6bmV4dC5vdXRwdXQ/PyIiLC4u"
    "LihuZXh0LmV4aXQ9PT11bmRlZmluZWQ/e306e2V4aXRfY29kZTpuZXh0LmV4aXR9KX07CiAgfQogIHMubGFzdFBvbGxSZXR1"
    "cm49cy50cmFjZS5sZW5ndGg7cmV0dXJuIHI7Cn0KZnVuY3Rpb24gYnJpZGdlKHMsb3BlcmF0aW9uKXsKICBjb25zdCByZW1l"
    "bWJlcj1lPT57aWYoRkFVTFRTLmhhcyhlKSYmcy5maXJzdEhvc3RGYXVsdD09PW51bGwpcy5maXJzdEhvc3RGYXVsdD1lO3Ro"
    "cm93IGU7fTsKICB0cnl7Y29uc3Qgdj1vcGVyYXRpb24oKTtyZXR1cm4gdiBpbnN0YW5jZW9mIFByb21pc2U/di5jYXRjaChy"
    "ZW1lbWJlcik6djt9CiAgY2F0Y2goZSl7cmV0dXJuIHJlbWVtYmVyKGUpO30KfQpmdW5jdGlvbiB0b29scyhzKXsKICBjb25z"
    "dCBtZXRob2RzPXsKICAgIGV4ZWNfY29tbWFuZDphPT5sb2NhbChzLGEpLHdyaXRlX3N0ZGluOmE9PnBvbGwocyxhKSwKICAg"
    "IG1jcF9fY3VhX2RyaXZlcl9fYnJvd3Nlcl9uYXZpZ2F0ZTphc3luYyBhPT57CiAgICAgIGJvdW5kKHMsYSk7ZW5zdXJlKFsi"
    "aHR0cDovL2xvY2FsaG9zdDozMDAwLyIsImh0dHA6Ly9sb2NhbGhvc3Q6MzAwMC9wcm90ZWN0ZWQiXS5pbmNsdWRlcyhhLnVy"
    "bCksCiAgICAgICAgIm5hdmlnYXRpb25fYWxsb3dsaXN0Iik7dHJhY2UocywibmF2aWdhdGUiKTtyZXR1cm4ge3N0cnVjdHVy"
    "ZWRDb250ZW50OntzdGF0dXM6Im9rIn19OwogICAgfSwKICAgIG1jcF9fY3VhX2RyaXZlcl9fZ2V0X2Jyb3dzZXJfc3RhdGU6"
    "YXN5bmMgYT0+ewogICAgICBib3VuZChzLGEpO2Vuc3VyZShhLnNuYXBzaG90X2Zvcm1hdD09PSJzZW1hbnRpY192MiImJmEu"
    "aW5jbHVkZV9zY3JlZW5zaG90PT09ZmFsc2UsCiAgICAgICAgInNuYXBzaG90X3B1YmxpY19vbmx5Iik7dHJhY2Uocywic25h"
    "cHNob3QiKTsKICAgICAgY29uc3Qga2luZD1zLnNuYXBzaG90UXVldWUuc2hpZnQoKT8/ImdlbmVyaWMiO2NvbnN0IHJlZj0i"
    "cDE6IitTdHJpbmcoKytzLnJlZkNvdW50ZXIpOwogICAgICBpZihraW5kPT09Im1pc3NpbmciKXJldHVybiB7c3RydWN0dXJl"
    "ZENvbnRlbnQ6e3N0YXR1czoib2siLHNuYXBzaG90Ontjb21wbGV0ZTp0cnVlfSwKICAgICAgICBjb250ZW50X3JlZnM6W10s"
    "cmVmczpbXX19OwogICAgICByZXR1cm4gcHVibGljUGFnZShraW5kLHJlZik7CiAgICB9LAogICAgbWNwX19jdWFfZHJpdmVy"
    "X19icm93c2VyX2NsaWNrOmFzeW5jIGE9PnsKICAgICAgYm91bmQocyxhLHRydWUpO2Vuc3VyZShhLmlucHV0X3JvdXRlPT09"
    "ImRvbV9ldmVudCIsImNsaWNrX3JvdXRlIik7CiAgICAgIHRyYWNlKHMsImNsaWNrIik7cy5pbnB1dEtpbmRzLnB1c2goImNs"
    "aWNrIik7cmV0dXJuIHtzdHJ1Y3R1cmVkQ29udGVudDp7ZWZmZWN0OiJjb25maXJtZWQifX07CiAgICB9LAogICAgbWNwX19j"
    "dWFfZHJpdmVyX19icm93c2VyX3R5cGU6YXN5bmMgYT0+ewogICAgICBib3VuZChzLGEsdHJ1ZSk7ZW5zdXJlKGEucmVwbGFj"
    "ZT09PXRydWUsInR5cGVfcmVwbGFjZSIpOwogICAgICBjb25zdCBraW5kPWEudGV4dD09PVBBU1NXT1JEPyJwYXNzd29yZCI6"
    "YS50ZXh0PT09ImFkbWluIj8idXNlcm5hbWUiOm51bGw7CiAgICAgIGVuc3VyZShraW5kIT09bnVsbCwidHlwZV92YWx1ZSIp"
    "O3RyYWNlKHMsInR5cGUiLHtraW5kLHBhc3N3b3JkX21hdGNoZXM6a2luZD09PSJwYXNzd29yZCJ9KTsKICAgICAgcy5pbnB1"
    "dEtpbmRzLnB1c2goa2luZCk7cmV0dXJuIHtzdHJ1Y3R1cmVkQ29udGVudDp7ZWZmZWN0OiJjb25maXJtZWQifX07CiAgICB9"
    "LAogICAgbWNwX19jdWFfZHJpdmVyX19raWxsX2FwcDphc3luYyBhPT57CiAgICAgIGVuc3VyZShhLnBpZD09PTEzLCJraWxs"
    "X2JvdW5kX293bmVkX3BpZCIpO3RyYWNlKHMsImtpbGwiKTsKICAgICAgaWYocy5vcHRpb25zLmRyaXZlcl9leGNlcHRpb24p"
    "dGhyb3cgbmV3IEVycm9yKEVYQ0VQVElPTik7CiAgICAgIHJldHVybiB7c3RydWN0dXJlZENvbnRlbnQ6e3N0YXR1czoib2si"
    "fX07CiAgICB9LAogICAgbWNwX19jdWFfZHJpdmVyX19lbmRfc2Vzc2lvbjphc3luYyBhPT57CiAgICAgIGVuc3VyZShhLnNl"
    "c3Npb249PT0ic3ludGhldGljLXNlc3Npb24iLCJlbmRfYm91bmRfc2Vzc2lvbiIpO3RyYWNlKHMsImVuZF9zZXNzaW9uIik7"
    "CiAgICAgIHJldHVybiB7c3RydWN0dXJlZENvbnRlbnQ6e2FjdGl2ZTpmYWxzZSxzZXNzaW9uOmEuc2Vzc2lvbn19OwogICAg"
    "fSwKICAgIG1jcF9fY3VhX2RyaXZlcl9fbGlzdF93aW5kb3dzOmFzeW5jIGE9PnsKICAgICAgZW5zdXJlKGEucGlkPT09MTMs"
    "IndpbmRvd3NfYm91bmRfcGlkIik7dHJhY2Uocywid2luZG93cyIpOwogICAgICByZXR1cm4ge3N0cnVjdHVyZWRDb250ZW50"
    "Ont3aW5kb3dzOltdfX07CiAgICB9CiAgfTsKICByZXR1cm4gT2JqZWN0LmZyZWV6ZShPYmplY3QuZnJvbUVudHJpZXMoT2Jq"
    "ZWN0LmVudHJpZXMobWV0aG9kcykKICAgIC5tYXAoKFtuYW1lLGZdKT0+W25hbWUsYT0+YnJpZGdlKHMsKCk9PmYoYSkpXSkp"
    "KTsKfQphc3luYyBmdW5jdGlvbiBjZWxsKHMpewogIGNsb2NrR3VhcmQoKTtlbnN1cmUocmVzdWx0LmNlbGxzPE1BWF9DRUxM"
    "UywiY2VsbF9jYXAiKTtyZXN1bHQuY2VsbHMrKzsKICBjb25zdCBzYW5kYm94PU9iamVjdC5jcmVhdGUobnVsbCk7CiAgT2Jq"
    "ZWN0LmFzc2lnbihzYW5kYm94LHt0b29sczp0b29scyhzKSxzdG9yZTooayx2KT0+YnJpZGdlKHMsKCk9PnN0b3JlKHMsayx2"
    "KSksCiAgICBsb2FkOms9PmJyaWRnZShzLCgpPT5sb2FkKHMsaykpLHRleHQ6dj0+YnJpZGdlKHMsKCk9Pntwcml2YWN5KHYp"
    "O3Mub3V0cHV0cy5wdXNoKGNsb25lKHYpKTt9KSwKICAgIGV4aXQ6KCk9Pnt0aHJvdyBFWElUO30sRGF0ZTpPYmplY3QuZnJl"
    "ZXplKHtub3c6KCk9PnMubm93fSl9KTsKICBjb25zdCBjdHg9dm0uY3JlYXRlQ29udGV4dChzYW5kYm94LHtjb2RlR2VuZXJh"
    "dGlvbjp7c3RyaW5nczpmYWxzZSx3YXNtOmZhbHNlfX0pOwogIGxldCB0aW1lcjsKICB0cnl7CiAgICBjb25zdCByZW1haW5p"
    "bmc9TWF0aC5tYXgoMSxNYXRoLmZsb29yKExJTUlUX01TLXBlcmZvcm1hbmNlLm5vdygpKSk7CiAgICBjb25zdCBleGVjdXRp"
    "b249U0NSSVBULnJ1bkluQ29udGV4dChjdHgse3RpbWVvdXQ6cmVtYWluaW5nLGRpc3BsYXlFcnJvcnM6ZmFsc2V9KTsKICAg"
    "IGF3YWl0IFByb21pc2UucmFjZShbUHJvbWlzZS5yZXNvbHZlKGV4ZWN1dGlvbiksbmV3IFByb21pc2UoKF8scmVqZWN0KT0+"
    "ewogICAgICB0aW1lcj1zZXRUaW1lb3V0KCgpPT5yZWplY3QoZmFpbHVyZSgicmVhbF9kZWFkbGluZSIpKSxyZW1haW5pbmcp"
    "OwogICAgfSldKTsKICB9Y2F0Y2goZSl7aWYoZSE9PUVYSVQpdGhyb3cgZTt9CiAgZmluYWxseXtpZih0aW1lciE9PXVuZGVm"
    "aW5lZCljbGVhclRpbWVvdXQodGltZXIpO30KICBjbG9ja0d1YXJkKCk7aWYocy5maXJzdEhvc3RGYXVsdCE9PW51bGwpdGhy"
    "b3cgcy5maXJzdEhvc3RGYXVsdDsKICBlbnN1cmUocy5vdXRwdXRzLmxlbmd0aD4wLCJjZWxsX291dHB1dCIpOwogIGZvcihj"
    "b25zdCBbayx2XSBvZiBzLm1hcClpZihrIT09UFcpcHJpdmFjeSh2KTsKICBwcml2YWN5KHMub3V0cHV0cyk7cHJpdmFjeShz"
    "Lm1ldGFkYXRhKTtwcml2YWN5KHMudHJhY2UpOwogIHJldHVybiBzLm91dHB1dHMuYXQoLTEpOwp9CmZ1bmN0aW9uIGZpbmFs"
    "T3V0Y29tZShzLGV4cGVjdGVkLHJlbGVhc2UpewogIGNvbnN0IG89cy5vdXRwdXRzLmF0KC0xKTtwcml2YWN5KG8pOwogIGVu"
    "c3VyZShvLndob2xlX2NsZWFudXBfd2l0aGluNjBfcHJvdmVuPT09ZmFsc2UsIm5vX3dob2xlNjBfY3JlZGl0Iik7CiAgZW5z"
    "dXJlKG8uam91cm5leV9jcmVkaXQhPT10cnVlLCJub19qb3VybmV5X2NyZWRpdCIpOwogIGlmKGV4cGVjdGVkPT09bnVsbCll"
    "bnN1cmUoby5yZXN1bHQ9PT0iYnJvd3Nlcl9kZWNpc2lvbl9vYnNlcnZlZF9vbmx5Iiwib2JzZXJ2ZWRfb25seSIpOwogIGVs"
    "c2UgZW5zdXJlKG8ucmVzdWx0PT09ImZhaWxlZCImJm8uZmlyc3RfZmFpbHVyZT09PWV4cGVjdGVkLCJmaXJzdF9mYWlsdXJl"
    "X21hdGNoZXMiKTsKICBpZihyZWxlYXNlIT09dW5kZWZpbmVkKWVuc3VyZShvLnJlc291cmNlX3JlbGVhc2VfcHJvdmVuPT09"
    "cmVsZWFzZSwicmVsZWFzZV9zZXBhcmF0ZSIpOwogIGlmKHMuc3RvcENvdW50KXtlbnN1cmUocy5zdG9wQ291bnQ9PT0xJiZz"
    "LnN0b3BQcm9vZnMubGVuZ3RoPT09MSwic2luZ2xlX2NsZWFudXBfcHJvdG9jb2wiKTsKICAgIGVuc3VyZShsb2FkKHMsUFcp"
    "PT09bnVsbCwicGFzc3dvcmRfY2xlYXJlZCIpO30KfQpmdW5jdGlvbiBmaXJzdEFjdGl2ZShzLG91dHB1dD0iIixleHRyYT17"
    "fSl7cy5xdWV1ZS5wdXNoKHtvdXRwdXQsLi4uZXh0cmF9KTt9CmZ1bmN0aW9uIHNldFRpbWUocyxtcyl7cy5ub3c9MTAwMDAw"
    "MCttczt9CmZ1bmN0aW9uIHBhcnRpYWxFdmVudChsZW5ndGgpewogIGNvbnN0IGZpeGVkPXt1bmV4cGVjdGVkX2ZhaWx1cmVf"
    "b2JzZXJ2YXRpb246e29ic2VydmVkOnRydWUsZGlhZ25vc3RpYzpudWxsfSwKICAgIHVuZXhwZWN0ZWRfb2JzZXJ2YXRpb25f"
    "cHJvamVjdGlvbjoidmFsaWQiLG9wYXF1ZTpQQVJUSUFMLHBhZDoiIn07CiAgbGV0IGxpbmU9ZXZlbnQoZml4ZWQpO2lmKGxl"
    "bmd0aCE9PXVuZGVmaW5lZCl7Zml4ZWQucGFkPSJ4Ii5yZXBlYXQobGVuZ3RoLWxpbmUubGVuZ3RoKTtsaW5lPWV2ZW50KGZp"
    "eGVkKTt9CiAgZW5zdXJlKGxlbmd0aD09PXVuZGVmaW5lZHx8bGluZS5sZW5ndGg9PT1sZW5ndGgsImZpeHR1cmVfbGluZV9z"
    "aXplIik7CiAgcmV0dXJuIGxpbmU7Cn0KYXN5bmMgZnVuY3Rpb24gcnVuQ2FzZShuYW1lKXsKICBsZXQgczsKICBzd2l0Y2go"
    "bmFtZSl7CiAgICBjYXNlICJwaGFzZV9lbnRyeV90aGVuX2NvbnRpbnVhdGlvbiI6ewogICAgICBzPW1ha2VTdGF0ZSh7ZW50"
    "cnk6dHJ1ZX0pO3Muc25hcHNob3RRdWV1ZT1bImJlZm9yZSIsImFwcGxpY2F0aW9uIl07CiAgICAgIGZpcnN0QWN0aXZlKHMs"
    "ZXZlbnQoe2ZpeHR1cmVfcmVhZHk6dHJ1ZSxndWFyZF9waWQ6MTEsc2VydmVyX3BpZDoxMixoZWxwZXJfcGlkOjE0LGxhYjoi"
    "L3N5bnRoZXRpYy9sYWIifSkpOwogICAgICBhd2FpdCBjZWxsKHMpO2Vuc3VyZShsb2FkKHMsT1dOKS5waGFzZT09PSJicm93"
    "c2VyX2RlY2lzaW9uIiYmbG9hZChzLE9XTikuaGVscGVyX3BpZD09PTE0LAogICAgICAgICJlbnRyeV9waGFzZV9hbmRfaGVs"
    "cGVyIik7ZW5zdXJlKHMucmVjZWlwdFJlYWR5UHJvb2ZzPT09MSwicmVhZHlfcmVjZWlwdF9wcm92ZWQiKTsKICAgICAgY29u"
    "c3Qgc3RhcnQ9bG9hZChzLE9XTikuc3RhcnRfZXBvY2hfbXM7ZGVjaXNpb24ocywic25hcHNob3QiKTthd2FpdCBjZWxsKHMp"
    "OwogICAgICBlbnN1cmUobG9hZChzLE9XTikuc3RhcnRfZXBvY2hfbXM9PT1zdGFydCwic3RhcnR1cF9idWRnZXRfcmV0YWlu"
    "ZWQiKTtmaW5hbE91dGNvbWUocyxudWxsLGZhbHNlKTticmVhazsKICAgIH0KICAgIGNhc2UgInBhc3N3b3JkX3N1cnZpdmVz"
    "X3VudGlsX3Bhc3N3b3JkX2Rpc3BhdGNoIjp7CiAgICAgIHM9bWFrZVN0YXRlKHtwYXNzd29yZDp0cnVlfSk7CiAgICAgIGZv"
    "cihjb25zdCBraW5kIG9mIFsidXNlcm5hbWUiLCJjbGljayIsInNuYXBzaG90Il0pewogICAgICAgIGNvbnN0IHJlZj1sb2Fk"
    "KHMsT1dOKS5mcmVzaF9yZWZzWzBdO2RlY2lzaW9uKHMsa2luZCxraW5kPT09InNuYXBzaG90Ij91bmRlZmluZWQ6cmVmKTsK"
    "ICAgICAgICBhd2FpdCBjZWxsKHMpO2Vuc3VyZShsb2FkKHMsUFcpPT09UEFTU1dPUkQmJnMuY2xlYXJFdmVudHMubGVuZ3Ro"
    "PT09MCwic2VjcmV0X3N1cnZpdmVzX25vbnBhc3N3b3JkIik7CiAgICAgIH0KICAgICAgZGVjaXNpb24ocywicGFzc3dvcmQi"
    "LGxvYWQocyxPV04pLmZyZXNoX3JlZnNbMF0pO2F3YWl0IGNlbGwocyk7CiAgICAgIGVuc3VyZShzLmlucHV0S2luZHMuam9p"
    "bigiLCIpPT09InVzZXJuYW1lLGNsaWNrLHBhc3N3b3JkIiwiaW5wdXRfcm91dGVfc2VxdWVuY2UiKTsKICAgICAgZW5zdXJl"
    "KGxvYWQocyxQVyk9PT1udWxsJiZzLmNsZWFyRXZlbnRzLmxlbmd0aD09PTEsInBhc3N3b3JkX2Rpc3BhdGNoX2NvbnN1bWVz"
    "X29uY2UiKTsKICAgICAgZmluYWxPdXRjb21lKHMsbnVsbCxmYWxzZSk7YnJlYWs7CiAgICB9CiAgICBjYXNlICJtaXNzaW5n"
    "X3Bhc3N3b3JkX3JlZnVzZXNfd2l0aG91dF9pbnB1dCI6ewogICAgICBzPW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInBhc3N3"
    "b3JkIiwicDE6MSIpO2F3YWl0IGNlbGwocyk7CiAgICAgIGVuc3VyZShzLmlucHV0S2luZHMubGVuZ3RoPT09MCwibm9faW5w"
    "dXRfb25fcmVmdXNhbCIpOwogICAgICBmaW5hbE91dGNvbWUocywiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsdHJ1"
    "ZSk7YnJlYWs7CiAgICB9CiAgICBjYXNlICJ1bm93bmVkX2NvbnRleHRfcmVmdXNlc193aXRob3V0X29wZXJhdGlvbnMiOnsK"
    "ICAgICAgcz1tYWtlU3RhdGUoKTtjb25zdCBvd249bG9hZChzLE9XTik7b3duLmRyaXZlcl9vd25lZD1mYWxzZTtzLm1hcC5z"
    "ZXQoT1dOLG93bik7CiAgICAgIGF3YWl0IGNlbGwocyk7ZW5zdXJlKHMudHJhY2UubGVuZ3RoPT09MCYmcy5vdXRwdXRzLmF0"
    "KC0xKS5wcm9wb3NhbF9yZWZ1c2VkPT09CiAgICAgICAgImZyZXNoX293bmVkX2NvbnRleHRfcmVxdWlyZWQiLCJ1bm93bmVk"
    "X3JlZnVzYWwiKTticmVhazsKICAgIH0KICAgIGNhc2UgInVuZGVjbGFyZWRfcmVmX3JlZnVzZXNfaW5wdXQiOnsKICAgICAg"
    "cz1tYWtlU3RhdGUoe3Bhc3N3b3JkOnRydWV9KTtkZWNpc2lvbihzLCJjbGljayIsInAxOjk5OSIpO2F3YWl0IGNlbGwocyk7"
    "CiAgICAgIGVuc3VyZShzLmlucHV0S2luZHMubGVuZ3RoPT09MCwibm9faW5wdXRfb25fcmVmdXNhbCIpOwogICAgICBmaW5h"
    "bE91dGNvbWUocywiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsdHJ1ZSk7YnJlYWs7CiAgICB9CiAgICBjYXNlICJz"
    "dGFsZV9yZWZfY2Fubm90X2Nyb3NzX2NlbGxzIjp7CiAgICAgIHM9bWFrZVN0YXRlKCk7ZGVjaXNpb24ocywiY2xpY2siLCJw"
    "MToxIik7YXdhaXQgY2VsbChzKTsKICAgICAgZW5zdXJlKCFsb2FkKHMsT1dOKS5mcmVzaF9yZWZzLmluY2x1ZGVzKCJwMTox"
    "IiksImZyZXNoX3JlZl9yZXBsYWNlc19jb25zdW1lZCIpOwogICAgICBkZWNpc2lvbihzLCJjbGljayIsInAxOjEiKTthd2Fp"
    "dCBjZWxsKHMpOwogICAgICBlbnN1cmUocy5pbnB1dEtpbmRzLmxlbmd0aD09PTEsInNpbmdsZV91c2VfcmVmIik7ZmluYWxP"
    "dXRjb21lKHMsImJyb3dzZXJfZGVjaXNpb25fdW5jb25maXJtZWQiLHRydWUpO2JyZWFrOwogICAgfQogICAgY2FzZSAiaW52"
    "YWxpZF9raW5kX3JlcGVhdGVkX2NsZWFudXBfY2xlYXJzX2JlZm9yZV9hd2FpdCI6ewogICAgICBzPW1ha2VTdGF0ZSh7cGFz"
    "c3dvcmQ6dHJ1ZX0pO2RlY2lzaW9uKHMsIm5vdC1kZWNsYXJlZCIpO2F3YWl0IGNlbGwocyk7CiAgICAgIGVuc3VyZShzLmNs"
    "ZWFyRXZlbnRzLmxlbmd0aD09PTImJnMuc3RvcENvdW50PT09MSwicmVwZWF0X2NsZWFyX25vX3JlcGVhdF9hd2FpdCIpOwog"
    "ICAgICBmaW5hbE91dGNvbWUocywiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsdHJ1ZSk7YnJlYWs7CiAgICB9CiAg"
    "ICBjYXNlICJoZWxwZXJfemVyb19maW5hbF9zbmFwc2hvdF90aGVuX2NsZWFudXAiOgogICAgY2FzZSAiaGVscGVyX3plcm9f"
    "bWlzc2luZ19wYWdlX3N0aWxsX2ZhaWxzIjoKICAgIGNhc2UgImhlbHBlcl9ub256ZXJvX2ZpbmFsX3N0aWxsX3JlZnVzZXMi"
    "OgogICAgY2FzZSAiaGVscGVyX3plcm9fb3RoZXJfa2luZF9zdGlsbF9yZWZ1c2VzIjp7CiAgICAgIHM9bWFrZVN0YXRlKCk7"
    "CiAgICAgIGNvbnN0IG90aGVyPW5hbWU9PT0iaGVscGVyX3plcm9fb3RoZXJfa2luZF9zdGlsbF9yZWZ1c2VzIjsKICAgICAg"
    "Y29uc3Qgbm9uemVybz1uYW1lPT09ImhlbHBlcl9ub256ZXJvX2ZpbmFsX3N0aWxsX3JlZnVzZXMiOwogICAgICBkZWNpc2lv"
    "bihzLG90aGVyPyJjbGljayI6InByb3RlY3RlZF9hZnRlciIsb3RoZXI/InAxOjEiOnVuZGVmaW5lZCk7CiAgICAgIHMuc25h"
    "cHNob3RRdWV1ZT1bbmFtZT09PSJoZWxwZXJfemVyb19taXNzaW5nX3BhZ2Vfc3RpbGxfZmFpbHMiPyJtaXNzaW5nIjoiYWZ0"
    "ZXIiXTsKICAgICAgZmlyc3RBY3RpdmUocyxldmVudCh7aGVscGVyX2NvbXBsZXRlZDp0cnVlLGV4aXQ6bm9uemVybz8xOjB9"
    "KSk7YXdhaXQgY2VsbChzKTsKICAgICAgY29uc3Qgc25hcHNob3RzPXMudHJhY2UuZmlsdGVyKHg9Pngua2luZD09PSJzbmFw"
    "c2hvdCIpLmxlbmd0aDsKICAgICAgZW5zdXJlKHMuaW5wdXRLaW5kcy5sZW5ndGg9PT0wJiYhcy50cmFjZS5zb21lKHg9Pngu"
    "a2luZD09PSJuYXZpZ2F0ZSIpLCJyZWFkX29ubHlfaGFuZG9mZiIpOwogICAgICBpZihub256ZXJvfHxvdGhlcil7ZW5zdXJl"
    "KHNuYXBzaG90cz09PTAsInplcm9fb25seV9kZWNsYXJlZF9maW5hbCIpOwogICAgICAgIGZpbmFsT3V0Y29tZShzLG5vbnpl"
    "cm8/ImhlbHBlcl9mYWlsZWQiOiJoZWxwZXJfY29tcGxldGVkX2JlZm9yZV9hcHBfY2hlY2twb2ludCIsdHJ1ZSk7fQogICAg"
    "ICBlbHNlIHtlbnN1cmUoc25hcHNob3RzPT09MSwib25lX2ZpbmFsX3NuYXBzaG90Iik7CiAgICAgICAgaWYobmFtZT09PSJo"
    "ZWxwZXJfemVyb19taXNzaW5nX3BhZ2Vfc3RpbGxfZmFpbHMiKWZpbmFsT3V0Y29tZShzLCJicm93c2VyX2RlY2lzaW9uX3Vu"
    "Y29uZmlybWVkIix0cnVlKTsKICAgICAgICBlbHNlIHtlbnN1cmUobG9hZChzLE9CUykucHJvdGVjdGVkX2FmdGVyX3BhZ2U9"
    "PT10cnVlLCJmaW5hbF9wYWdlX3Byb3ZlZCIpO2ZpbmFsT3V0Y29tZShzLG51bGwsdHJ1ZSk7fX0KICAgICAgYnJlYWs7CiAg"
    "ICB9CiAgICBjYXNlICJoZWxwZXJfemVyb19wcmVwYXJlZF9waGFzZV9zdGlsbF9yZWZ1c2VzIjp7CiAgICAgIHM9bWFrZVN0"
    "YXRlKHtlbnRyeTp0cnVlfSk7ZGVjaXNpb24ocywicHJvdGVjdGVkX2FmdGVyIik7CiAgICAgIGZpcnN0QWN0aXZlKHMsZXZl"
    "bnQoe2ZpeHR1cmVfcmVhZHk6dHJ1ZSxndWFyZF9waWQ6MTEsc2VydmVyX3BpZDoxMixoZWxwZXJfcGlkOjE0LAogICAgICAg"
    "IGxhYjoiL3N5bnRoZXRpYy9sYWIiLGhlbHBlcl9jb21wbGV0ZWQ6dHJ1ZSxleGl0OjB9KSk7YXdhaXQgY2VsbChzKTsKICAg"
    "ICAgZW5zdXJlKCFzLnRyYWNlLnNvbWUoeD0+WyJuYXZpZ2F0ZSIsInNuYXBzaG90IiwidHlwZSIsImNsaWNrIl0uaW5jbHVk"
    "ZXMoeC5raW5kKSksInByZXBhcmVkX3BoYXNlX3JlZnVzYWwiKTsKICAgICAgZmluYWxPdXRjb21lKHMsImhlbHBlcl9jb21w"
    "bGV0ZWRfYmVmb3JlX2FwcF9jaGVja3BvaW50Iix0cnVlKTticmVhazsKICAgIH0KICAgIGNhc2UgInBhcnRpYWxfbGluZV9k"
    "cmFpbnNfYmVmb3JlX291dHB1dF9hbmRfbmV4dF9jZWxsIjoKICAgIGNhc2UgInBhcnRpYWxfZXhhY3RfY2FwX2lzX2FjY2Vw"
    "dGVkIjp7CiAgICAgIHM9bWFrZVN0YXRlKCk7ZGVjaXNpb24ocywic25hcHNob3QiKTsKICAgICAgY29uc3QgbGluZT1wYXJ0"
    "aWFsRXZlbnQobmFtZT09PSJwYXJ0aWFsX2V4YWN0X2NhcF9pc19hY2NlcHRlZCI/MTYzODQ6dW5kZWZpbmVkKTsKICAgICAg"
    "Zmlyc3RBY3RpdmUocyk7Zmlyc3RBY3RpdmUocyxsaW5lLnNsaWNlKDAsLTEpKTtmaXJzdEFjdGl2ZShzLGxpbmUuc2xpY2Uo"
    "LTEpKTthd2FpdCBjZWxsKHMpOwogICAgICBlbnN1cmUocy5wb2xsQ291bnQ9PT0zJiZsb2FkKHMsT0JTKS51bmV4cGVjdGVk"
    "X2ZhaWx1cmVfb2JzZXJ2YXRpb24/Lm9ic2VydmVkPT09dHJ1ZSwKICAgICAgICAicGFydGlhbF9ldmVudF9wcm9jZXNzZWRf"
    "YmVmb3JlX291dHB1dCIpOwogICAgICBlbnN1cmUoWy4uLnMubWFwLmtleXMoKV0uc29ydCgpLmpvaW4oIiwiKT09PVtPV04s"
    "T0JTXS5zb3J0KCkuam9pbigiLCIpLCJub19wYXJ0aWFsX3N0b3JlIik7CiAgICAgIGZpbmFsT3V0Y29tZShzLG51bGwsZmFs"
    "c2UpOwogICAgICBpZihuYW1lPT09InBhcnRpYWxfbGluZV9kcmFpbnNfYmVmb3JlX291dHB1dF9hbmRfbmV4dF9jZWxsIil7"
    "CiAgICAgICAgY29uc3QgcHJldmlvdXM9cy5wb2xsQ291bnQ7ZGVjaXNpb24ocywic25hcHNob3QiKTthd2FpdCBjZWxsKHMp"
    "OwogICAgICAgIGVuc3VyZShzLnBvbGxDb3VudC1wcmV2aW91cz09PTIsIm5vX3BhcnRpYWxfY2FycnlfbmV4dF9jZWxsIik7"
    "ZmluYWxPdXRjb21lKHMsbnVsbCxmYWxzZSk7CiAgICAgIH1icmVhazsKICAgIH0KICAgIGNhc2UgInBhcnRpYWxfb3Zlcl9j"
    "YXBfcmVmdXNlcyI6ewogICAgICBzPW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInNuYXBzaG90Iik7Zmlyc3RBY3RpdmUocyk7"
    "Zmlyc3RBY3RpdmUocyxwYXJ0aWFsRXZlbnQoMTYzODUpKTsKICAgICAgYXdhaXQgY2VsbChzKTtmaW5hbE91dGNvbWUocywi"
    "Y29udHJvbGxlcl9vYnNlcnZhdGlvbl9pbnZhbGlkIix0cnVlKTticmVhazsKICAgIH0KICAgIGNhc2UgInBhcnRpYWxfam9p"
    "bmVkX2NvbnRyb2xsZXJfcmVmdXNlcyI6ewogICAgICBzPW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInNuYXBzaG90Iik7Zmly"
    "c3RBY3RpdmUocyk7Zmlyc3RBY3RpdmUocywneyJvcGFxdWUiOiInK1BBUlRJQUwse2V4aXQ6MH0pOwogICAgICBhd2FpdCBj"
    "ZWxsKHMpO2ZpbmFsT3V0Y29tZShzLCJjb250cm9sbGVyX2NvbXBsZXRlZF9iZWZvcmVfYXBwX2NoZWNrcG9pbnQiLHRydWUp"
    "O2JyZWFrOwogICAgfQogICAgY2FzZSAicGFydGlhbF91bmF2YWlsYWJsZV9jb250cm9sbGVyX3JlZnVzZXMiOnsKICAgICAg"
    "cz1tYWtlU3RhdGUoKTtkZWNpc2lvbihzLCJzbmFwc2hvdCIpO2ZpcnN0QWN0aXZlKHMsJ3sib3BhcXVlIjoiJytQQVJUSUFM"
    "KTsKICAgICAgcy5xdWV1ZS5wdXNoKHt0aHJvdzp0cnVlfSk7YXdhaXQgY2VsbChzKTsKICAgICAgZmluYWxPdXRjb21lKHMs"
    "ImNvbnRyb2xsZXJfb2JzZXJ2YXRpb25fdW5hdmFpbGFibGUiLGZhbHNlKTticmVhazsKICAgIH0KICAgIGNhc2UgInBhcnRp"
    "YWxfZGVhZGxpbmVfcHJldmVudHNfZXh0cmFfcG9sbCI6ewogICAgICBzPW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInNuYXBz"
    "aG90Iik7Zmlyc3RBY3RpdmUocyk7CiAgICAgIGZpcnN0QWN0aXZlKHMsJ3sib3BhcXVlIjoiJytQQVJUSUFMLHthdDo4NDAw"
    "MDB9KTthd2FpdCBjZWxsKHMpOwogICAgICBlbnN1cmUocy5wb2xsQ291bnQ9PT0zJiZzLnRyYWNlLmZpbHRlcih4PT54Lmtp"
    "bmQ9PT0iY29udHJvbGxlcl9wb2xsIiYmIXguY2xlYW51cCkubGVuZ3RoPT09MiwKICAgICAgICAibm9fZXh0cmFfYWN0aXZl"
    "X3BvbGxfYXRfZGVhZGxpbmUiKTsKICAgICAgZmluYWxPdXRjb21lKHMsImJyb3dzZXJfYWN0aXZlX2RlYWRsaW5lIix0cnVl"
    "KTticmVhazsKICAgIH0KICAgIGNhc2UgInBhcnRpYWxfY29tcGxldGVkX2xhdGVfc3RpbGxfcmVmdXNlcyI6ewogICAgICBz"
    "PW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInNuYXBzaG90Iik7Y29uc3QgbGluZT1wYXJ0aWFsRXZlbnQoKTsKICAgICAgZmly"
    "c3RBY3RpdmUocyk7Zmlyc3RBY3RpdmUocyxsaW5lLnNsaWNlKDAsLTEpLHthdDo4Mzk5OTl9KTsKICAgICAgZmlyc3RBY3Rp"
    "dmUocyxsaW5lLnNsaWNlKC0xKSx7YXQ6ODQwMDAwfSk7YXdhaXQgY2VsbChzKTsKICAgICAgZmluYWxPdXRjb21lKHMsImJy"
    "b3dzZXJfYWN0aXZlX2RlYWRsaW5lIix0cnVlKTticmVhazsKICAgIH0KICAgIGNhc2UgInJldGFpbmVkX3N0YXJ0X2J1ZGdl"
    "dF9yZWZ1c2VzX25leHRfY2VsbCI6ewogICAgICBzPW1ha2VTdGF0ZSgpO2RlY2lzaW9uKHMsInNuYXBzaG90Iik7YXdhaXQg"
    "Y2VsbChzKTtjb25zdCBvPWxvYWQocyxPV04pOwogICAgICBzZXRUaW1lKHMsODQwMDAwKTtkZWNpc2lvbihzLCJzbmFwc2hv"
    "dCIpO2NvbnN0IGJlZm9yZT1zLnRyYWNlLmZpbHRlcih4PT54LmtpbmQ9PT0ic25hcHNob3QiKS5sZW5ndGg7CiAgICAgIGF3"
    "YWl0IGNlbGwocyk7ZW5zdXJlKGxvYWQocyxPV04pLnN0YXJ0X2Vwb2NoX21zPT09by5zdGFydF9lcG9jaF9tcywic3RhcnR1"
    "cF9idWRnZXRfcmV0YWluZWQiKTsKICAgICAgZW5zdXJlKHMudHJhY2UuZmlsdGVyKHg9Pngua2luZD09PSJzbmFwc2hvdCIp"
    "Lmxlbmd0aD09PWJlZm9yZSwibm9fc25hcHNob3RfYWZ0ZXJfZGVhZGxpbmUiKTsKICAgICAgZmluYWxPdXRjb21lKHMsImJy"
    "b3dzZXJfYWN0aXZlX2RlYWRsaW5lIix0cnVlKTticmVhazsKICAgIH0KICAgIGNhc2UgImluY2x1c2l2ZV9jbGVhbnVwX2Rl"
    "YWRsaW5lX2RvZXNfbm90X2ludmVudF9qb2luIjp7CiAgICAgIHM9bWFrZVN0YXRlKCk7ZGVjaXNpb24ocywic25hcHNob3Qi"
    "KTtzZXRUaW1lKHMsOTAwMDAwKTthd2FpdCBjZWxsKHMpOwogICAgICBlbnN1cmUobG9hZChzLE9CUykuY29udHJvbGxlcl9l"
    "eGl0PT09bnVsbCwibm9fam9pbl9jcmVkaXRfYXRfaW5jbHVzaXZlX2xpbWl0Iik7CiAgICAgIGZpbmFsT3V0Y29tZShzLCJi"
    "cm93c2VyX2FjdGl2ZV9kZWFkbGluZSIsZmFsc2UpO2JyZWFrOwogICAgfQogICAgY2FzZSAiZmlyc3RfcGFnZV9mYWlsdXJl"
    "X3N1cnZpdmVzX2xhdGVyX2hlbHBlcl9lcnJvciI6CiAgICBjYXNlICJtaXNzaW5nX2Fic2VuY2VfcHJvb2ZfcHJldmVudHNf"
    "cmVsZWFzZSI6CiAgICBjYXNlICJjbGVhbnVwX2V4Y2VwdGlvbl9pc19wcml2YXRlIjoKICAgIGNhc2UgInBlbmRpbmdfbWV0"
    "YWRhdGFfY29tbWFuZF9wcmV2ZW50c19yZWxlYXNlIjp7CiAgICAgIHM9bWFrZVN0YXRlKHtjbGVhbnVwX2hlbHBlcl9lcnJv"
    "cjpuYW1lPT09ImZpcnN0X3BhZ2VfZmFpbHVyZV9zdXJ2aXZlc19sYXRlcl9oZWxwZXJfZXJyb3IiLAogICAgICAgIGFic2Vu"
    "Y2VfbWlzc2luZzpuYW1lPT09Im1pc3NpbmdfYWJzZW5jZV9wcm9vZl9wcmV2ZW50c19yZWxlYXNlIiwKICAgICAgICBkcml2"
    "ZXJfZXhjZXB0aW9uOm5hbWU9PT0iY2xlYW51cF9leGNlcHRpb25faXNfcHJpdmF0ZSIsCiAgICAgICAgbWV0YWRhdGFfcGVu"
    "ZGluZzpuYW1lPT09InBlbmRpbmdfbWV0YWRhdGFfY29tbWFuZF9wcmV2ZW50c19yZWxlYXNlIixwYXNzd29yZDp0cnVlfSk7"
    "CiAgICAgIGRlY2lzaW9uKHMsInByb3RlY3RlZF9hZnRlciIpO3Muc25hcHNob3RRdWV1ZT1bIm1pc3NpbmciXTthd2FpdCBj"
    "ZWxsKHMpOwogICAgICBmaW5hbE91dGNvbWUocywiYnJvd3Nlcl9kZWNpc2lvbl91bmNvbmZpcm1lZCIsCiAgICAgICAgbmFt"
    "ZSE9PSJtaXNzaW5nX2Fic2VuY2VfcHJvb2ZfcHJldmVudHNfcmVsZWFzZSImJm5hbWUhPT0icGVuZGluZ19tZXRhZGF0YV9j"
    "b21tYW5kX3ByZXZlbnRzX3JlbGVhc2UiKTsKICAgICAgaWYobmFtZT09PSJjbGVhbnVwX2V4Y2VwdGlvbl9pc19wcml2YXRl"
    "IillbnN1cmUobG9hZChzLE9CUykuY2xlYW51cF9lcnJvcnMuaW5jbHVkZXMoCiAgICAgICAgImRyaXZlcl9vcGVyYXRpb25f"
    "dW5jb25maXJtZWQiKSwiZHJpdmVyX2ZhaWx1cmVfcmV0YWluZWQiKTsKICAgICAgYnJlYWs7CiAgICB9CiAgICBkZWZhdWx0"
    "OnRocm93IGZhaWx1cmUoInVua25vd25fY2FzZSIpOwogIH0KICBwcml2YWN5KHMub3V0cHV0cyk7cHJpdmFjeShzLm1ldGFk"
    "YXRhKTtwcml2YWN5KHMudHJhY2UpOwp9CmFzeW5jIGZ1bmN0aW9uIG1haW4oKXsKICBsZXQgY3VycmVudD1udWxsOwogIHRy"
    "eXsKICAgIGNsb2NrR3VhcmQoKTtiaW5kU291cmNlKCk7CiAgICBlbnN1cmUoQklORElORy5jYXNlX3BsYW4ubGVuZ3RoPT09"
    "bmV3IFNldChCSU5ESU5HLmNhc2VfcGxhbi5tYXAoeD0+eC5uYW1lKSkuc2l6ZSwidW5pcXVlX2Nhc2VfbmFtZXMiKTsKICAg"
    "IGZvcihjb25zdCByb3cgb2YgQklORElORy5jYXNlX3BsYW4pewogICAgICBjbG9ja0d1YXJkKCk7Y3VycmVudD1yb3cubmFt"
    "ZTtyZXN1bHQuYXR0ZW1wdGVkLnB1c2gocm93Lm5hbWUpOwogICAgICBhd2FpdCBydW5DYXNlKHJvdy5uYW1lKTtyZXN1bHQu"
    "Y29tcGxldGVkLnB1c2gocm93Lm5hbWUpOwogICAgICByZXN1bHQuY29tcGxldGVkX2dyb3Vwc1tyb3cuZ3JvdXBdPShyZXN1"
    "bHQuY29tcGxldGVkX2dyb3Vwc1tyb3cuZ3JvdXBdfHwwKSsxOwogICAgfQogIH1jYXRjaChlKXsKICAgIGNvbnN0IGNvZGU9"
    "KGUhPT1udWxsJiYodHlwZW9mIGU9PT0ib2JqZWN0Inx8dHlwZW9mIGU9PT0iZnVuY3Rpb24iKSYmRkFVTFRTLmhhcyhlKSk/"
    "CiAgICAgIEZBVUxUUy5nZXQoZSk6InVuZXhwZWN0ZWQiOwogICAgcmVzdWx0LmZpcnN0X2ZhaWx1cmU9e2Nhc2U6Y3VycmVu"
    "dCxjaGVjazpCSU5ESU5HLmNoZWNrX25hbWVzLmluY2x1ZGVzKGNvZGUpP2NvZGU6InVuZXhwZWN0ZWQifTsKICB9CiAgcmVz"
    "dWx0LnVucmVhY2hlZD1CSU5ESU5HLmNhc2VfcGxhbi5tYXAoeD0+eC5uYW1lKS5maWx0ZXIoeD0+IXJlc3VsdC5hdHRlbXB0"
    "ZWQuaW5jbHVkZXMoeCkpOwogIHJlc3VsdC5lbGFwc2VkX21zPXBlcmZvcm1hbmNlLm5vdygpOwogIGlmKHJlc3VsdC5lbGFw"
    "c2VkX21zPj1MSU1JVF9NUyYmcmVzdWx0LmZpcnN0X2ZhaWx1cmU9PT1udWxsKQogICAgcmVzdWx0LmZpcnN0X2ZhaWx1cmU9"
    "e2Nhc2U6bnVsbCxjaGVjazoicmVhbF9kZWFkbGluZSJ9OwogIC8vIE9ubHkgdGhlIGNsb3NlZCBmaW5pdGUgcGFja2V0IGlz"
    "IHdyaXR0ZW47IG5vIHJhdyBleGNlcHRpb24vdHJhY2Uvc291cmNlL3N0dWIgdmFsdWUuCiAgY29uc3Qgb3V0cHV0PUpTT04u"
    "c3RyaW5naWZ5KHJlc3VsdCk7CiAgaWYoUFJJVkFURS5zb21lKHg9Pm91dHB1dC5pbmNsdWRlcyh4KSkpewogICAgcHJvY2Vz"
    "cy5zdGRvdXQud3JpdGUoSlNPTi5zdHJpbmdpZnkoe3NjaGVtYTpyZXN1bHQuc2NoZW1hLGZpcnN0X2ZhaWx1cmU6e2Nhc2U6"
    "bnVsbCxjaGVjazoicHJpdmFjeV9zaW5rIn19KSsiXG4iKTsKICAgIHByb2Nlc3MuZXhpdENvZGU9MTtyZXR1cm47CiAgfQog"
    "IHByb2Nlc3Muc3Rkb3V0LndyaXRlKG91dHB1dCsiXG4iKTtwcm9jZXNzLmV4aXRDb2RlPXJlc3VsdC5maXJzdF9mYWlsdXJl"
    "PT09bnVsbD8wOjE7Cn0KYXdhaXQgbWFpbigpOwo="
)
PLAN=json.loads("[{\"name\":\"phase_entry_then_continuation\",\"group\":\"phase_binding\"},{\"name\":\"password_survives_until_password_dispatch\",\"group\":\"secret_lifetime\"},{\"name\":\"missing_password_refuses_without_input\",\"group\":\"secret_lifetime\"},{\"name\":\"unowned_context_refuses_without_operations\",\"group\":\"phase_binding\"},{\"name\":\"undeclared_ref_refuses_input\",\"group\":\"phase_binding\"},{\"name\":\"stale_ref_cannot_cross_cells\",\"group\":\"phase_binding\"},{\"name\":\"invalid_kind_repeated_cleanup_clears_before_await\",\"group\":\"secret_lifetime\"},{\"name\":\"helper_zero_final_snapshot_then_cleanup\",\"group\":\"helper_handoff\"},{\"name\":\"helper_zero_missing_page_still_fails\",\"group\":\"helper_handoff\"},{\"name\":\"helper_nonzero_final_still_refuses\",\"group\":\"helper_handoff\"},{\"name\":\"helper_zero_other_kind_still_refuses\",\"group\":\"helper_handoff\"},{\"name\":\"helper_zero_prepared_phase_still_refuses\",\"group\":\"helper_handoff\"},{\"name\":\"partial_line_drains_before_output_and_next_cell\",\"group\":\"partial_framing\"},{\"name\":\"partial_exact_cap_is_accepted\",\"group\":\"partial_framing\"},{\"name\":\"partial_over_cap_refuses\",\"group\":\"partial_framing\"},{\"name\":\"partial_joined_controller_refuses\",\"group\":\"partial_framing\"},{\"name\":\"partial_unavailable_controller_refuses\",\"group\":\"partial_framing\"},{\"name\":\"partial_deadline_prevents_extra_poll\",\"group\":\"partial_framing\"},{\"name\":\"partial_completed_late_still_refuses\",\"group\":\"partial_framing\"},{\"name\":\"retained_start_budget_refuses_next_cell\",\"group\":\"phase_binding\"},{\"name\":\"inclusive_cleanup_deadline_does_not_invent_join\",\"group\":\"cleanup_latch\"},{\"name\":\"first_page_failure_survives_later_helper_error\",\"group\":\"cleanup_latch\"},{\"name\":\"missing_absence_proof_prevents_release\",\"group\":\"cleanup_latch\"},{\"name\":\"cleanup_exception_is_private\",\"group\":\"cleanup_latch\"},{\"name\":\"pending_metadata_command_prevents_release\",\"group\":\"cleanup_latch\"}]")
CHECK_NAMES=json.loads("[\"real_deadline\",\"source_encoding\",\"privacy_sink\",\"diff_framing\",\"diff_headers\",\"diff_hunk\",\"diff_position\",\"diff_line\",\"diff_exact_line\",\"diff_hunk_count\",\"candidate_identity\",\"baseline_identity\",\"diff_identity\",\"edit_unique\",\"baseline_inverse\",\"collector_identity\",\"collector_in_candidate\",\"collector_set\",\"trace_cap\",\"call_cap\",\"store_key\",\"password_store_value\",\"receipt_precedes_comparison\",\"ready_receipt_order\",\"load_key\",\"collector_command\",\"collector_quoting\",\"collector_allowlist\",\"collector_arguments\",\"marker_bound\",\"clear_before_cleanup_await\",\"latch_before_cleanup\",\"clear_survives_cleanup_await\",\"readback_bound\",\"persist_bound\",\"bound_browser_handles\",\"bound_reference\",\"bound_controller\",\"navigation_allowlist\",\"snapshot_public_only\",\"click_route\",\"type_replace\",\"type_value\",\"kill_bound_owned_pid\",\"end_bound_session\",\"windows_bound_pid\",\"cell_cap\",\"cell_output\",\"no_whole60_credit\",\"no_journey_credit\",\"observed_only\",\"first_failure_matches\",\"release_separate\",\"single_cleanup_protocol\",\"password_cleared\",\"fixture_line_size\",\"entry_phase_and_helper\",\"ready_receipt_proved\",\"startup_budget_retained\",\"secret_survives_nonpassword\",\"input_route_sequence\",\"password_dispatch_consumes_once\",\"no_input_on_refusal\",\"unowned_refusal\",\"fresh_ref_replaces_consumed\",\"single_use_ref\",\"repeat_clear_no_repeat_await\",\"read_only_handoff\",\"zero_only_declared_final\",\"one_final_snapshot\",\"final_page_proved\",\"prepared_phase_refusal\",\"partial_event_processed_before_output\",\"no_partial_store\",\"no_partial_carry_next_cell\",\"no_extra_active_poll_at_deadline\",\"no_snapshot_after_deadline\",\"no_join_credit_at_inclusive_limit\",\"driver_failure_retained\",\"unique_case_names\",\"unexpected\",\"metadata_cap\"]")
EXPECTED_SOURCE=json.loads("{\"candidate_sha256\":\"7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8\",\"candidate_bytes\":32502,\"baseline_sha256\":\"d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d\",\"baseline_bytes\":31581,\"diff_sha256\":\"395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144\",\"logic_sha256\":\"639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054\",\"full_inverse\":true}")
CHILD_DEADLINE=START_NS+30_000_000_000
OUTER_DEADLINE=START_NS+35_000_000_000
STDOUT_CAP=65536
STDERR_CAP=16384
FLOOR=8*1024**3
first_failure=None
def latch(tag):
    global first_failure
    if first_failure is None:first_failure=tag
def within(deadline):
    return time.monotonic_ns()<deadline
def assemble_payload():
    raw=base64.b64decode(PAYLOAD_B64,validate=True)
    if len(raw)!=PAYLOAD_BYTES or hashlib.sha256(raw).hexdigest()!=PAYLOAD_SHA:
        raise ValueError("payload_identity")
    raw.decode("utf-8")
    return raw
def regular_node():
    if NODE.is_symlink():raise ValueError("node_identity")
    info=NODE.stat()
    if not stat.S_ISREG(info.st_mode) or not os.access(NODE,os.X_OK):raise ValueError("node_identity")
    with NODE.open("rb") as source:
        h=hashlib.sha256()
        while True:
            if not within(CHILD_DEADLINE):raise TimeoutError()
            part=source.read(1024*1024)
            if not part:break
            h.update(part)
    if h.hexdigest()!=NODE_SHA:raise ValueError("node_identity")
def private_directory(parent,name):
    fd=os.open(parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        info=os.fstat(fd)
        if info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o700:
            raise ValueError("private_directory")
        os.mkdir(name,0o700,dir_fd=fd)
    finally:os.close(fd)
    path=parent/name
    info=path.lstat()
    if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
        raise ValueError("private_directory")
    return path
def save(parent,name,raw):
    if len(raw)>262144:raise ValueError("evidence_cap")
    fd=os.open(parent/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        os.fchmod(fd,0o600)
        with os.fdopen(fd,"wb",closefd=False) as out:
            out.write(raw);out.flush();os.fsync(out.fileno())
    finally:os.close(fd)
def encode(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True)+"\n").encode("ascii")
def duplicate_free(pairs):
    result={}
    for key,value in pairs:
        if key in result:raise ValueError("duplicate_json")
        result[key]=value
    return result
def true_int(value,low,high):
    return type(value) is int and low<=value<=high
def closed_packet(raw):
    if not raw.endswith(b"\n") or raw.count(b"\n")!=1:raise ValueError("packet_framing")
    packet=json.loads(raw.decode("utf-8"),object_pairs_hook=duplicate_free,
                      parse_constant=lambda value:(_ for _ in ()).throw(ValueError("json_constant")))
    keys={"schema","source","planned","attempted","completed","completed_groups","cells",
          "stub_counts","assertions_completed","privacy_checks_completed","first_failure",
          "unreached","elapsed_ms","full_candidate_only","actual_tools_or_product"}
    if type(packet) is not dict or set(packet)!=keys:raise ValueError("packet_schema")
    if packet["schema"]!="riauth.d01-continuation-memory/v1" or packet["planned"]!=PLAN:
        raise ValueError("packet_plan")
    names=[row["name"] for row in PLAN]
    for key in ("attempted","completed","unreached"):
        values=packet[key]
        if type(values) is not list or len(values)>len(names) or any(type(v) is not str for v in values):
            raise ValueError("packet_cases")
    attempted,completed=packet["attempted"],packet["completed"]
    if attempted!=names[:len(attempted)] or completed!=names[:len(completed)]:
        raise ValueError("packet_prefix")
    if len(completed)>len(attempted) or len(attempted)-len(completed)>1:
        raise ValueError("packet_progress")
    if packet["unreached"]!=names[len(attempted):]:raise ValueError("packet_unreached")
    groups={}
    group_by_name={row["name"]:row["group"] for row in PLAN}
    for name in completed:
        group=group_by_name[name];groups[group]=groups.get(group,0)+1
    if type(packet["completed_groups"]) is not dict or packet["completed_groups"]!=groups:
        raise ValueError("packet_groups")
    if any(type(v) is not int for v in packet["completed_groups"].values()):
        raise ValueError("packet_groups")
    if not true_int(packet["cells"],0,75):raise ValueError("packet_cells")
    for key in ("assertions_completed","privacy_checks_completed"):
        if not true_int(packet[key],0,50000):raise ValueError("packet_counts")
    allowed={"collector_clock","collector_stop","collector_readback","collector_marker","collector_persist",
             "controller_poll","navigate","snapshot","click","type","kill","end_session","windows"}
    counts=packet["stub_counts"]
    if type(counts) is not dict or not set(counts)<=allowed or any(
            not true_int(v,1,2500) for v in counts.values()) or sum(counts.values())>2500:
        raise ValueError("packet_stub_counts")
    if packet["full_candidate_only"] is not True or packet["actual_tools_or_product"] is not False:
        raise ValueError("packet_scope")
    elapsed=packet["elapsed_ms"]
    if type(elapsed) not in (int,float) or not math.isfinite(elapsed) or not 0<=elapsed<=30000:
        raise ValueError("packet_elapsed")
    failure=packet["first_failure"]
    if failure is not None:
        if type(failure) is not dict or set(failure)!={"case","check"}:raise ValueError("packet_failure")
        if failure["case"] is not None and failure["case"] not in names:raise ValueError("packet_failure")
        if type(failure["check"]) is not str or failure["check"] not in CHECK_NAMES:
            raise ValueError("packet_failure")
    if packet["source"] is not None:
        source=packet["source"]
        if type(source) is not dict or set(source)!=set(EXPECTED_SOURCE):
            raise ValueError("packet_source")
        if any(type(source[k]) is not type(v) or source[k]!=v for k,v in EXPECTED_SOURCE.items()):
            raise ValueError("packet_source")
    return packet
def group_cleanup(child):
    # WNOWAIT keeps the owned leader unreaped until this exact group is signaled.
    reaped=False;empty=False;code=None
    try:
        observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if os.getpgid(child.pid)!=child.pid:raise ValueError("group_identity")
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        remaining=max(0.001,(OUTER_DEADLINE-time.monotonic_ns())/1e9)
        code=child.wait(timeout=remaining);reaped=True
        try:os.killpg(child.pid,0)
        except ProcessLookupError:empty=True
        if not empty:latch("owned_group_not_empty")
    except Exception:
        latch("owned_cleanup_unconfirmed")
        # Never signal any newly discovered PID or process group.
    return code,reaped,empty
def main():
    evidence=None;child=None;selector=None;raw_out=bytearray();raw_err=bytearray()
    code=None;reaped=False;empty=False;packet=None;grade=False;spawn_count=0
    minimum_free=None;last_disk=0;sent=0;exit_seen=False;phase="preflight"
    stream_eof={"out":False,"err":False};output_capped=False
    try:
        if len(sys.argv)!=2 or re.fullmatch(r"[0-9a-f]{16}",sys.argv[1]) is None:
            raise ValueError("input")
        if pathlib.Path.cwd()!=ROOT or ROOT.is_symlink():raise ValueError("workspace")
        private=ROOT/"deployment-private"
        info=private.lstat()
        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
            raise ValueError("private_directory")
        if not within(CHILD_DEADLINE):raise TimeoutError()
        if not all(hasattr(os,name) for name in ("waitid","P_PID","WEXITED","WNOHANG","WNOWAIT")):
            raise ValueError("owned_wait_unavailable")
        payload=assemble_payload();regular_node()
        free=shutil.disk_usage(ROOT).free;minimum_free=free
        if free<FLOOR:raise ValueError("disk_floor")
        evidence=private_directory(private,"d01-continuation-memory-"+sys.argv[1])
        phase="spawn"
        child=subprocess.Popen([str(NODE),"--input-type=module","-"],cwd=evidence,
            env={"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"},
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            start_new_session=True,bufsize=0)
        spawn_count=1
        selector=selectors.DefaultSelector()
        for stream,kind,events in ((child.stdin,"in",selectors.EVENT_WRITE),
                                   (child.stdout,"out",selectors.EVENT_READ),
                                   (child.stderr,"err",selectors.EVENT_READ)):
            os.set_blocking(stream.fileno(),False);selector.register(stream,events,kind)
        phase="child"
        while selector.get_map() or not exit_seen:
            now=time.monotonic_ns()
            if now>=CHILD_DEADLINE:latch("child_deadline");break
            if now-last_disk>=2_000_000_000:
                free=shutil.disk_usage(ROOT).free;minimum_free=min(minimum_free,free);last_disk=now
                if free<FLOOR:latch("disk_floor");break
            observed=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            if observed is not None:exit_seen=True
            for key,events in selector.select(min(.02,max(0,(CHILD_DEADLINE-now)/1e9))):
                stream,kind=key.fileobj,key.data
                if kind=="in":
                    try:written=os.write(stream.fileno(),payload[sent:sent+16384])
                    except BrokenPipeError:
                        latch("child_input_closed");selector.unregister(stream);stream.close();continue
                    sent+=written
                    if sent==len(payload):selector.unregister(stream);stream.close()
                else:
                    try:part=os.read(stream.fileno(),4096)
                    except BlockingIOError:continue
                    if not part:selector.unregister(stream);stream.close();continue
                    target,cap=(raw_out,STDOUT_CAP) if kind=="out" else (raw_err,STDERR_CAP)
                    if len(target)+len(part)>cap:
                        target.extend(part[:cap-len(target)]);output_capped=True;latch("child_output_cap");break
                    target.extend(part)
            if first_failure is not None:break
        if not exit_seen:
            observed=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            exit_seen=observed is not None
        if not exit_seen and first_failure is None:latch("child_exit_unconfirmed")
    except Exception:
        latch("controller_"+phase+"_failed")
    finally:
        if child is not None:
            code,reaped,empty=group_cleanup(child)
            if selector is not None:
                # Drain only already available bytes; no wait, no unbounded read.
                for key in list(selector.get_map().values()):
                    stream,kind=key.fileobj,key.data
                    if kind in ("out","err"):
                        target,cap=(raw_out,STDOUT_CAP) if kind=="out" else (raw_err,STDERR_CAP)
                        while len(target)<=cap:
                            try:part=os.read(stream.fileno(),min(4096,cap-len(target)+1))
                            except (BlockingIOError,OSError):break
                            if not part:stream_eof[kind]=True;break
                            if len(target)+len(part)>cap:
                                target.extend(part[:cap-len(target)]);output_capped=True;latch("child_output_cap");break
                            target.extend(part)
                    try:selector.unregister(stream)
                    except Exception:pass
                    stream.close()
                selector.close()
    # Actual exit and full bounded output are fsynced/closed BEFORE parsing/grading.
    receipt={"schema":"riauth.d01-continuation-memory-retained/v1","spawn_count":spawn_count,
             "child_pid":None if child is None else child.pid,"child_exit":code,"child_reaped":reaped,
             "owned_group_empty":empty,"first_failure_before_grade":first_failure,
             "payload_bytes":PAYLOAD_BYTES,"payload_sha256":PAYLOAD_SHA,
             "stdout_bytes":len(raw_out),"stdout_sha256":hashlib.sha256(raw_out).hexdigest(),
             "stderr_bytes":len(raw_err),"stderr_sha256":hashlib.sha256(raw_err).hexdigest(),
             "minimum_free_bytes":minimum_free,"stream_eof":stream_eof,"output_capped":output_capped}
    retained=False;review_closed=False
    try:
        if evidence is None:raise ValueError("no_evidence_directory")
        save(evidence,"child.stdout",bytes(raw_out));save(evidence,"child.stderr",bytes(raw_err))
        save(evidence,"retained.json",encode(receipt))
        retained=all(stream_eof.values()) and not output_capped
        if not retained:latch("child_output_incomplete")
        try:packet=closed_packet(bytes(raw_out))
        except Exception:latch("child_packet_invalid")
        if code!=0:latch("child_nonzero")
        if raw_err:latch("child_stderr_nonempty")
        if not reaped or not empty:latch("owned_cleanup_unconfirmed")
        if packet is not None:
            if packet["source"]!=EXPECTED_SOURCE:latch("child_source_missing")
            if packet["first_failure"] is not None:latch("child_case_failed")
            if packet["completed"]!=[row["name"] for row in PLAN]:latch("child_incomplete")
            if not packet["cells"] or not packet["assertions_completed"] or not packet["privacy_checks_completed"]:
                latch("child_counts_missing")
        grade=first_failure is None
        # This journal is explicitly provisional until the final clock below.
        save(evidence,"review.json",encode({"schema":"riauth.d01-continuation-memory-review/v1",
             "full_packet_retained_before_grade":retained,"grade_before_final_clock":grade,
             "first_failure":first_failure,"closed_child_packet":packet,
             "child_exit":code,"child_reaped":reaped,"owned_group_empty":empty}))
        review_closed=True
    except Exception:latch("evidence_unconfirmed")
    final_ns=time.monotonic_ns()
    # NO evidence file write/fsync/close, directory mutation or owned-child operation follows.
    within35=final_ns<=OUTER_DEADLINE
    if not within35:latch("final_deadline")
    passed=grade and review_closed and retained and first_failure is None
    returned={"schema":"riauth.d01-continuation-memory-return/v1",
              "result":"passed" if passed else "failed","first_failure":first_failure,
              "child_exit":code,"child_reaped":reaped,"owned_group_empty":empty,
              "full_packet_retained_before_grade":retained,"review_closed":review_closed,
              "within35":within35,"elapsed_seconds":(final_ns-START_NS)/1e9,
              "completed_cases":None if packet is None else len(packet["completed"]),
              "child_failure":None if packet is None else packet["first_failure"],
              "actual_product_or_driver":False}
    sys.stdout.write(json.dumps(returned,sort_keys=True,separators=(",",":"))+"\n");sys.stdout.flush()
    return 0 if passed else 1
if __name__=="__main__":sys.exit(main())
```

### Actual static checks and unexecuted limits

Acorn parsed the complete future Node module and full async wrapper without
importing/evaluating that module. Static AST selectors verified: exact complete
candidate/baseline/diff identities and four-span inverse; wrapper body equals
the complete candidate AST; exactly one full-candidate vm.Script construction
and one fresh-context factory; only node:vm/node:crypto/node:perf_hooks imports;
no require/dynamic import; all 25 declared cases exactly match the plan; and
the finite check catalog exactly covers all static ensure labels plus unexpected.
The parser driver executes only its own data/AST comparison, not harness logic.

Python AST parsing and compile-only produced a code object for the complete
209948-byte controller WITHOUT executing its imports/functions/main. Literal
inspection verifies its embedded payload bytes/hash, case/check plan, expected
source identities and Node launcher pin. AST/source selectors verify one Popen,
persistent 30/35 deadlines, WNOWAIT ownership, retention-before-grade, final
journal close before final clock and no post-clock evidence/child operations.
All these final static checks exited 0; they are not VM/transport/cleanup runs.

One initial AST selector exited 1 for an unused draft wrapper declaration.
Removing that unused draft line/comment left the sole reviewed wrapper
construction; subsequent syntax/selector checks exited 0. An earlier static
assembly capture could not JSON-parse an oversized warning-prefixed tool
response; source assembly was changed to in-memory data encoding with
metadata-only hash output. Neither preparation issue executed any candidate,
VM, stub, case, controller or product. A source review also tightened the
controller's source-identity comparison to exact JSON types so numeric 1 cannot
stand in for full_inverse true; the final compiled archive above includes it.

The new full-cell Node VM async/Promise bridge, POSIX quote decoder, synthetic
tools/receipt model, all cases, real deadlines, EOF/exit control, mode/fsync
behavior and actual group cleanup are UNEXECUTED. Static parsing does not
establish that a future case passes or an environment supports every primitive.
Runtime/memory peak/free disk and full actual launch/delivery/return duration
remain unmeasured. Any unexpected future failure is a stop/report outcome,
not permission to fix a harness or rerun automatically.

Prior 18-case old-cell stub PASS, 128-case helper-memory PASS and 78 legacy
guard/header cases are distinct historical source scopes, not current candidate
credit. Historical c88 actual FAIL, zero Authorization count, protected-before
403, unknown sender/cause/provider values/cleanup anchor and unproved whole60
remain unchanged. Root's shared-host 370778 http_rates row and active native
X64 container 37078338496 are separate; neither is queried or used as evidence
for this design. No full D01, D05, release, tenant, application or status closure
follows. Root full/independent source review precedes one separately released
stub run; complete adapter source review precedes any separately released
Driver journey. Driver MCP remains the only future GUI provider, with
descriptions/current state inspected first.

Only this report is appended. Candidate/source/helper/controller/collector
files, guides, product/build/config/manifests, main, board, task ownership,
old reports and all 61754 prior bytes remain untouched. No worker contact,
new worker/task/WT/managed shell, merge/alignment/reset, push, allocation,
cache deletion or runtime slot occurs. Final immutable archive/prefix/scope/
docs/whitespace checks and the one report-only commit are in the handoff.

Final actual report checks: the entire 61754-byte prior prefix is exact;
all five source fences are closed and byte/hash-identical to their canonical
archives; the controller's decoded payload equals the complete readable
132725-byte Node archive. Acorn source/AST/binding selectors and Python
AST/compile-only selectors passed. The declared 25 case names/groups and
finite check catalog were derived/matched statically; executed cases and
allocated harness VM/controller/fixture resources remain zero. Direct
trailing-whitespace scan and `git diff --check` passed. The global docs
checker exited 1 only for the same five existing private target-directory
names listed in the preserved prefix, with no link failure. Only this report
has an append; no runtime or product/source outcome is certified. Final
immutable commit/file hash and clean worktree proof are in the handoff.
