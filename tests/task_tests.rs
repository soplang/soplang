use soplang::{run_source, ReplSession};

#[test]
fn compiled_tasks_return_values_and_can_wait_for_children() {
    run_source(
        r#"
        hawl labalaab(abn n) { celi n * 2 }
        hawl parent(abn n) {
            door ilmo = shaqo(labalaab, n)
            celi sug(ilmo) + 1
        }
        door task = shaqo(parent, 20)
        hubi(sug(task), 41)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn cancelled_task_raises_a_catchable_error() {
    run_source(
        r#"
        hawl value() { celi 1 }
        door task = shaqo(value)
        baaji(task)
        door caught = been
        tijaabi { sug(task) } qabo (e) { caught = run }
        hubi(caught)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn cancellation_resumes_suspended_task_through_cleanup() {
    run_source(
        r#"
        door cleaned = been
        hawl pause() {
            tijaabi {
                door child = shaqo(hawl() { celi 7 })
                sug(child)
            } dhammee { cleaned = run }
        }
        hawl stop(task) { baaji(task) }
        door target = shaqo(pause)
        door stopper = shaqo(stop, target)
        door caught = been
        tijaabi { sug(target) } qabo (e) { caught = run }
        hubi(caught)
        hubi(cleaned)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn collection_keeps_values_on_suspended_compiled_stacks() {
    run_source(
        r#"
        hawl allocate() {
            abn i = 0
            intay (i < 2200) {
                door temporary = [i]
                i = i + 1
            }
        }
        hawl parked() {
            door value = [99]
            sug(shaqo(allocate))
            hubi(value[0], 99)
        }
        sug(shaqo(parked))
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn completed_task_cycles_are_collected() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval(
            r#"
            door values = []
            door task = shaqo(hawl() { celi values })
            values.geli(task)
            sug(task)
            values = maran
            task = maran
            "#,
            None,
        )
        .unwrap();
    session.collect();
    assert_eq!(session.memory_stats().tasks, 0);
}

#[test]
fn loop_polls_let_ready_tasks_run() {
    run_source(
        r#"
        door ready = been
        hawl mark() { ready = run }
        door child = shaqo(mark)
        abn attempts = 0
        intay (!ready && attempts < 100) { attempts = attempts + 1 }
        hubi(ready)
        hubi(attempts < 100)
        sug(child)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn cpu_task_cancellation_runs_cleanup_at_a_loop_poll() {
    run_source(
        r#"
        door cleaned = been
        hawl spin() {
            tijaabi {
                abn i = 0
                intay (i < 1000000) { i = i + 1 }
            } dhammee { cleaned = run }
        }
        hawl stop(task) { baaji(task) }
        door worker = shaqo(spin)
        door stopper = shaqo(stop, worker)
        door caught = been
        tijaabi { sug(worker) } qabo (e) { caught = run }
        hubi(caught && cleaned)
        sug(stopper)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn timer_tasks_complete_without_blocking_other_tasks() {
    run_source(
        r#"
        door ran = been
        hawl mark() { ran = run }
        hawl wait() { sug(hurdada(20)) celi 7 }
        door timer = hurdada(20)
        door marker = shaqo(mark)
        door waiter = shaqo(wait)
        sug(timer)
        hubi(ran)
        hubi(sug(waiter), 7)
        sug(marker)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn cancelling_a_timer_wakes_awaiters_promptly() {
    let started = std::time::Instant::now();
    run_source(
        r#"
        door timer = hurdada(10000)
        baaji(timer)
        door caught = been
        tijaabi { sug(timer) } qabo (e) { caught = run }
        hubi(caught)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

#[test]
fn unawaited_timers_cancel_at_scope_exit_and_completed_handles_collect() {
    let started = std::time::Instant::now();
    run_source("door timer=hurdada(10000)", None, false, false, false).unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(2));

    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("abn i=0 intay (i<100) { sug(hurdada(0)) i=i+1 }", None)
        .unwrap();
    session.collect();
    assert_eq!(session.memory_stats().tasks, 0);
}
