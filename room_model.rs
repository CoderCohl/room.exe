use backrooms_terminal::room::{RoomState};

#[test]
fn room_state_enum_works() {
    assert_eq!(format!("{:?}", RoomState::ACTIVE), "ACTIVE");
}
