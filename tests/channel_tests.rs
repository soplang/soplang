use soplang::{run_source, ReplSession};

#[test]
fn buffered_channel_applies_backpressure_and_drains_after_close() {
    run_source(
        r#"
        door channel = marin(1)
        hawl produce() {
            dir(channel, 4)
            dir(channel, 5)
            xidh(channel)
        }
        door producer = shaqo(produce)
        door first = qaad(channel)
        door second = qaad(channel)
        door end = qaad(channel)
        hubi(first.qiime, 4)
        hubi(second.qiime, 5)
        hubi(!first.dhammaad && !second.dhammaad && end.dhammaad)
        sug(producer)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn rendezvous_channel_waits_for_receive_and_rejects_late_sends() {
    run_source(
        r#"
        door channel = marin(0)
        hawl produce() { dir(channel, 41) celi 9 }
        door producer = shaqo(produce)
        hubi(qaad(channel).qiime, 41)
        hubi(sug(producer), 9)
        xidh(channel)
        hubi(qaad(channel).dhammaad)
        door caught = been
        tijaabi { dir(channel, 1) } qabo (e) { caught = run }
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
fn cancelled_receiver_leaves_channel_usable() {
    run_source(
        r#"
        door channel = marin(0)
        hawl receive() { celi qaad(channel) }
        hawl stop(task) { baaji(task) }
        door receiver = shaqo(receive)
        door stopper = shaqo(stop, receiver)
        door caught = been
        tijaabi { sug(receiver) } qabo (e) { caught = run }
        hubi(caught)
        door producer = shaqo(hawl() { dir(channel, 8) })
        hubi(qaad(channel).qiime, 8)
        sug(producer)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn unreferenced_channel_cycles_are_collected() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval(
            r#"
            door channel = marin(1)
            door values = [channel]
            dir(channel, values)
            values = maran
            channel = maran
            "#,
            None,
        )
        .unwrap();
    session.collect();
    assert_eq!(session.memory_stats().channels, 0);
}

#[test]
fn two_tasks_exchange_a_long_bounded_stream() {
    run_source(
        r#"
        door channel = marin(3)
        hawl produce() {
            abn i = 0
            intay (i < 500) {
                dir(channel, i)
                i = i + 1
            }
            xidh(channel)
        }
        hawl consume() {
            abn total = 0
            intay (run) {
                door item = qaad(channel)
                haddii (item.dhammaad) { jooji }
                total = total + item.qiime
            }
            celi total
        }
        door producer = shaqo(produce)
        door consumer = shaqo(consume)
        hubi(sug(consumer), 124750)
        sug(producer)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn closing_wakes_blocked_receivers_and_senders() {
    run_source(
        r#"
        door empty = marin(1)
        door receiver = shaqo(hawl() { celi qaad(empty) })
        door closer = shaqo(hawl() { xidh(empty) })
        hubi(sug(receiver).dhammaad)
        sug(closer)

        door full = marin(1)
        dir(full, 1)
        door sender = shaqo(hawl() { dir(full, 2) })
        door closer2 = shaqo(hawl() { xidh(full) })
        door caught = been
        tijaabi { sug(sender) } qabo (e) { caught = run }
        hubi(caught)
        hubi(qaad(full).qiime, 1)
        hubi(qaad(full).dhammaad)
        sug(closer2)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}
