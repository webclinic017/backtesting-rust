pub mod vector_utils;
pub mod utils;
pub mod strategy;
pub mod events;


#[test]
fn playground_test() {
    // let v = [1,0,0,0,1,-1,0,0,-1,0,0,1,-1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,-1,0,0,0,0,1];

    // let v: Vec<Any> = ["hell", 9];
    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list.csv";
    let events: Vec<crate::events::Event> = crate::events::get_event_calendar(events_loc, &[3]);

    let e = &events[0];
    println!("{:?}", e.datetime());
}
