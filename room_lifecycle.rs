use backrooms_terminal::persistence::filesystem::FilesystemPersistence;
use backrooms_terminal::persistence::Persistence;
use backrooms_terminal::room::{Room, RoomConfig, RoomMetadata, RoomState};
use backrooms_terminal::memory::MemoryStore;
use backrooms_terminal::entity::EntityState;
use tempfile::tempdir;

#[test]
fn create_save_load_room() {
    let dir = tempdir().unwrap();
    let p = FilesystemPersistence::new(dir.path());
    p.init().unwrap();

    let now = 1_700_000_000i64;
    let room = Room{
        id: "test".to_string(),
        created_at: now,
        last_active: now,
        state: RoomState::ACTIVE,
        config: RoomConfig::default(),
        memory: MemoryStore::new(1024),
        entity_state: EntityState::default(),
        metadata: RoomMetadata{
            creation_timestamp: now,
            creator_pid: 1,
            creator_user: "u".to_string(),
            creator_host: "h".to_string(),
            total_inputs: 0,
            total_outputs: 0,
            total_errors: 0,
            last_error: None,
            state_version: 1,
        },
    };

    p.save_room(&room).unwrap();
    let loaded = p.load_room("test").unwrap();
    assert_eq!(loaded.id, "test");
}
