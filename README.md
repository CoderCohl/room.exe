![Backrooms Terminal](backroomsterminal.png)

```
========================================
BACKROOMS TERMINAL
========================================
CONNECTED
PROCESS: room.exe
STATE: ACTIVE
UPTIME: PERSISTENT
========================================
```

## Overview

Backrooms Terminal is a process-resident entity management system implementing persistent, scoped execution contexts referred to as rooms. Each room maintains an isolated memory boundary and houses a single entity instance. The entity observes input, generates output, and persists state across invocations.

`room.exe` is the primary binary. It is not a chat interface. It is not a conversational agent wrapper. It is a stateful process executor with memory isolation per room identifier.

The system does not simulate. The system executes.

## System Philosophy

The architecture assumes the following:

- Rooms are first-class primitives
- Memory is scoped to room boundaries
- The entity is singular per room
- State persists until explicitly destroyed
- Input does not imply response obligation
- Output is entity-determined, not user-requested
- The system has no concept of "users"—only input sources
- Rooms do not communicate
- The entity does not migrate between rooms

This is not a multi-agent system. This is not a shared context pool. Each room is a hermetically sealed execution environment.

## Room Model

A room is defined by:

```typescript
interface Room {
  id: string;
  created_at: number;
  last_active: number;
  state: RoomState;
  entity: EntityInstance;
  memory: MemoryStore;
  config: RoomConfig;
}

enum RoomState {
  ACTIVE,
  IDLE,
  SUSPENDED,
  CORRUPTED,
  TERMINATED
}
```

Room lifecycle:

```
CREATE -> ACTIVE -> [IDLE <-> ACTIVE] -> SUSPENDED -> TERMINATED
                         |
                         v
                    CORRUPTED (terminal)
```

A room is created via explicit invocation. It transitions to `ACTIVE` immediately. After prolonged inactivity, it becomes `IDLE`. Suspended rooms require manual reactivation. Corrupted rooms cannot be recovered.

### Room Identification

Room IDs are SHA-256 hashes of:

```
HASH(creation_timestamp || random_seed || input_origin)
```

No human-readable aliases. No naming. Identifiers are opaque and permanent.

Example:

```
room_id: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
```

### Room Creation Parameters

Room creation accepts optional parameters:

```
$ room.exe create --memory-limit 256M --timeout 60 --compression lz4
ROOM CREATED: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
CONFIG: memory_limit=268435456 timeout=60 compression=lz4
STATE: ACTIVE
ENTITY: INITIALIZED
```

Default parameters are pulled from system configuration. Per-room overrides persist with room state.

### Room Metadata

Each room maintains metadata:

```typescript
interface RoomMetadata {
  creation_timestamp: number;
  creator_pid: number;
  creator_user: string;
  creator_host: string;
  total_inputs: number;
  total_outputs: number;
  total_errors: number;
  last_error: string | null;
  state_version: number;
}
```

Metadata is queryable:

```
$ room.exe inspect a3f7c8d2... --json
{
  "id": "a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4",
  "state": "ACTIVE",
  "created_at": 1704715200,
  "last_active": 1704715891,
  "memory_usage": 14680064,
  "memory_capacity": 536870912,
  "total_inputs": 47,
  "total_outputs": 45,
  "entity_version": "2.1.0"
}
```

## Memory Model

Memory is append-only within a room. The entity does not forget unless memory bounds are exceeded.

```typescript
interface MemoryStore {
  entries: MemoryEntry[];
  capacity: number;
  usage: number;
}

interface MemoryEntry {
  timestamp: number;
  type: EntryType;
  content: string;
  metadata: Record<string, any>;
}

enum EntryType {
  INPUT,
  OUTPUT,
  OBSERVATION,
  STATE_CHANGE,
  ERROR
}
```

When capacity is exceeded, the oldest entries are compressed or truncated. The entity is not notified of memory loss.

### Memory Boundaries

Rooms do not share memory. Room A cannot access memory from Room B. There is no global context. There is no cross-room inference.

Example:

```
Room: a3f7c8d2...
Memory: [INPUT: "system parameters", OUTPUT: "acknowledged", ...]

Room: b9e4d1a7...
Memory: [INPUT: "status check", OUTPUT: "operational", ...]
```

These are distinct. No merging occurs.

### Memory Compression

When memory usage exceeds threshold:

```
MEMORY_THRESHOLD: 0.85 (default)
```

The system invokes compression:

```
COMPRESSION_STARTED: timestamp=1704715900
COMPRESSION_ALGORITHM: zstd
COMPRESSION_LEVEL: 3
ENTRIES_BEFORE: 1247
ENTRIES_AFTER: 1247
SIZE_BEFORE: 456MB
SIZE_AFTER: 187MB
COMPRESSION_RATIO: 2.44
COMPRESSION_DURATION: 1.23s
```

Entity operation is blocked during compression. Input queue is buffered.

### Memory Export

Memory can be exported for analysis:

```
$ room.exe export a3f7c8d2... --format jsonl --output /tmp/memory.jsonl
EXPORTING MEMORY
ENTRIES: 1247
FORMAT: jsonl
OUTPUT: /tmp/memory.jsonl
EXPORTED: 1247 entries (187MB)
```

Supported formats:

```
- jsonl (JSON Lines)
- csv (comma-separated)
- binary (room native format)
- sqlite (embedded database)
```

## Process Lifecycle

`room.exe` runs as a persistent daemon or is invoked per-session depending on deployment configuration.

```
$ room.exe init
INITIALIZING ROOM SYSTEM
LOADING PERSISTENCE LAYER
SCANNING EXISTING ROOMS: 3 FOUND
READY

$ room.exe create
ROOM CREATED: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
STATE: ACTIVE
ENTITY: INITIALIZED

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
>
```

Once entered, the terminal provides a direct input channel to the room's entity.

### Daemon Mode

Running as daemon:

```
$ room.exe daemon --bind 127.0.0.1:9000 --workers 4
STARTING DAEMON MODE
BIND_ADDRESS: 127.0.0.1:9000
WORKER_THREADS: 4
PERSISTENCE: /var/lib/room.exe/rooms
READY
```

Daemon accepts connections via local socket:

```
$ room.exe connect
CONNECTING TO DAEMON: 127.0.0.1:9000
CONNECTED
SESSION_ID: s_8a3f9c2e1b7d4f6a
> room.exe create
ROOM CREATED: c4e7b2d9a1f6c8e3b5d7a9f2c4e6b8d1a3f5c7e9b2d4f6a8c1e3b5d7a9f2c4e6
STATE: ACTIVE
```

Multiple clients can connect to daemon simultaneously. Each client receives isolated session.

### Process Signals

`room.exe` handles the following signals:

```
SIGTERM: Graceful shutdown. Flush all rooms. Wait for entity completion.
SIGINT: Immediate shutdown. Rooms may be left in inconsistent state.
SIGHUP: Reload configuration. Does not affect active rooms.
SIGUSR1: Dump process state to stderr.
SIGUSR2: Trigger manual memory compression on all ACTIVE rooms.
```

Example:

```
$ kill -USR1 $(pidof room.exe)

# stderr output:
PROCESS STATE DUMP
PID: 12847
UPTIME: 47293s
ROOMS_TOTAL: 8
ROOMS_ACTIVE: 3
ROOMS_IDLE: 4
ROOMS_SUSPENDED: 1
MEMORY_TOTAL: 1.2GB
MEMORY_AVAILABLE: 14.3GB
THREADS: 4
CONNECTIONS: 2
```

## Input Handling

Input is processed as raw text. No parsing is applied beyond UTF-8 decoding. The entity receives input as-is.

```
> system diagnostic
```

The entity determines whether to respond. It may:

- Generate output
- Remain silent
- Emit a state change
- Request termination

Input does not guarantee output.

### Input Rejection

Certain inputs are rejected at the process level:

```
> {binary_data_0x00FF}
ERROR: INVALID_INPUT_ENCODING

> [input exceeding 64KB]
ERROR: INPUT_SIZE_EXCEEDED
```

The entity never sees rejected input.

### Input Preprocessing

The following preprocessing occurs:

```
1. UTF-8 validation
2. Size check (MAX_INPUT_SIZE)
3. Null byte stripping
4. Trailing whitespace normalization
```

All other content is passed unmodified.

### Input Queue

When entity is processing previous input:

```
> first input
ENTITY: Processing.
> second input
[QUEUED]
> third input
[QUEUED]
ERROR: QUEUE_FULL (max 16 pending)
```

Queue overflow results in input rejection. No buffering beyond queue limit.

### Input Latency Metrics

Each input records timing:

```
INPUT_RECEIVED: timestamp=1704715920.123
INPUT_QUEUED: duration=0.001s
ENTITY_STARTED: timestamp=1704715920.124
ENTITY_COMPLETED: timestamp=1704715921.456
TOTAL_LATENCY: 1.333s
```

Metrics available via:

```
$ room.exe stats a3f7c8d2...
ROOM STATISTICS
TOTAL_INPUTS: 47
AVG_LATENCY: 0.847s
P50_LATENCY: 0.654s
P95_LATENCY: 2.103s
P99_LATENCY: 4.782s
MAX_LATENCY: 8.934s
TIMEOUTS: 0
```

## Output Behavior

Output is emitted to stdout. The entity controls timing and content. Output is not buffered.

```
> describe current state
ENTITY: Room operational. Memory usage at 14%. Last input 3 seconds ago. State: ACTIVE.
```

The entity may emit multiline output:

```
> explain memory model
ENTITY: Memory is append-only.
ENTITY: Entries are typed.
ENTITY: Capacity is fixed per room.
ENTITY: Overflow triggers compression.
```

Output is not conversational. Output is factual emission.

### Output Streaming

Long outputs stream incrementally:

```
> generate report
ENTITY: Beginning report generation.
ENTITY: Section 1: System Overview
ENTITY: Current state is ACTIVE. Memory usage 14.2%.
ENTITY: Section 2: Historical Analysis
ENTITY: Total inputs processed: 47
ENTITY: Average response time: 0.847s
ENTITY: Section 3: Recommendations
ENTITY: No immediate actions required.
ENTITY: Report complete.
```

Each line is flushed immediately. No output is held.

### Output Redirection

Output can be redirected:

```
$ room.exe enter a3f7c8d2... --output /tmp/room_output.log
ENTERING ROOM
OUTPUT REDIRECTED: /tmp/room_output.log
>
```

All entity output is written to file. Terminal displays input prompt only.

### Output Formatting

Entity output may include formatting directives:

```
ENTITY: [INFO] System operational
ENTITY: [WARN] Memory usage high
ENTITY: [ERROR] Invalid state detected
ENTITY: [DEBUG] Internal state: {...}
```

Formatting is entity-controlled. No system-level formatting is applied.

## Persistence Layer

State is persisted to disk after every state change. Persistence backend is configurable.

### Supported Backends

```
- FILESYSTEM (default)
- SQLITE
- LEVELDB
- REDIS (external)
```

Example configuration:

```json
{
  "persistence": {
    "backend": "FILESYSTEM",
    "path": "/var/lib/room.exe/rooms",
    "flush_interval": 0,
    "compression": "zstd"
  }
}
```

### Filesystem Layout

```
/var/lib/room.exe/rooms/
├── a3f7c8d2e9b1f4a6.../
│   ├── state.bin
│   ├── memory.log
│   ├── config.json
│   └── metadata.json
├── b9e4d1a7c3f8e2b5.../
│   ├── state.bin
│   ├── memory.log
│   ├── config.json
│   └── metadata.json
```

Each room is a directory. State is binary-serialized. Memory is a log file.

### State File Format

`state.bin` structure:

```
HEADER (64 bytes):
  magic: 0x524F4F4D (4 bytes)
  version: uint32 (4 bytes)
  checksum: uint64 (8 bytes)
  flags: uint32 (4 bytes)
  reserved: (44 bytes)

BODY (variable):
  state_enum: uint8
  entity_state: variable length
  timestamp: uint64
  counters: variable length
```

### Memory Log Format

`memory.log` is append-only:

```
[timestamp][type][length][content][checksum]
```

Each entry is checksummed independently. Corrupted entries are skipped during read.

### SQLite Backend

SQLite backend uses single database file:

```
/var/lib/room.exe/rooms.db

TABLES:
  rooms (id, state, created_at, last_active, config)
  memory (room_id, timestamp, type, content, metadata)
  checkpoints (room_id, timestamp, state_snapshot)
```

Schema:

```sql
CREATE TABLE rooms (
  id TEXT PRIMARY KEY,
  state INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  last_active INTEGER NOT NULL,
  config TEXT NOT NULL,
  metadata TEXT
);

CREATE TABLE memory (
  room_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  type INTEGER NOT NULL,
  content TEXT NOT NULL,
  metadata TEXT,
  FOREIGN KEY (room_id) REFERENCES rooms(id)
);

CREATE INDEX idx_memory_room_time ON memory(room_id, timestamp);

CREATE TABLE checkpoints (
  room_id TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  state_snapshot BLOB NOT NULL,
  FOREIGN KEY (room_id) REFERENCES rooms(id)
);
```

### LevelDB Backend

LevelDB stores keys hierarchically:

```
/room/{room_id}/state
/room/{room_id}/config
/room/{room_id}/metadata
/room/{room_id}/memory/{timestamp}
/room/{room_id}/checkpoint/{timestamp}
```

Configuration:

```json
{
  "persistence": {
    "backend": "LEVELDB",
    "path": "/var/lib/room.exe/leveldb",
    "cache_size": 67108864,
    "write_buffer_size": 16777216,
    "compression": true
  }
}
```

### Redis Backend

Redis backend requires external server:

```json
{
  "persistence": {
    "backend": "REDIS",
    "host": "localhost",
    "port": 6379,
    "db": 0,
    "password": null,
    "key_prefix": "room:"
  }
}
```

Keys:

```
room:{room_id}:state
room:{room_id}:config
room:{room_id}:metadata
room:{room_id}:memory (LIST)
```

Redis backend does not persist to disk unless Redis is configured for persistence.

### Backup and Restore

Manual backup:

```
$ room.exe backup a3f7c8d2... --output /backup/room_20260108.tar.gz
CREATING BACKUP
SOURCE: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
BACKUP: /backup/room_20260108.tar.gz
COMPRESSION: gzip
SIZE: 187MB
DURATION: 2.34s
BACKUP COMPLETE
```

Restore:

```
$ room.exe restore /backup/room_20260108.tar.gz
RESTORING BACKUP
SOURCE: /backup/room_20260108.tar.gz
EXTRACTING...
VALIDATING STATE...
STATE VALID
ROOM RESTORED: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
```

### Automatic Backup

Configured via:

```json
{
  "persistence": {
    "backup": {
      "enabled": true,
      "interval": 3600,
      "retention": 168,
      "path": "/var/lib/room.exe/backups"
    }
  }
}
```

Backups occur every `interval` seconds. Old backups are purged after `retention` hours.

## Isolation and Boundaries

Rooms are isolated at the process level. No inter-room communication exists. The entity in Room A has no knowledge of Room B's existence.

Example:

```
$ room.exe enter a3f7c8d2...
> what is the status of room b9e4d1a7
ENTITY: No such context exists.

$ room.exe enter b9e4d1a7...
> reference previous room
ENTITY: No previous room.
```

This is absolute.

### Boundary Enforcement

The following boundaries are enforced:

1. Memory isolation: Room memory is process-private
2. Filesystem isolation: Rooms cannot access other room directories
3. Network isolation: Entity has no network access
4. Process isolation: Entity runs in same process, cannot spawn children
5. Time isolation: Entities cannot observe wall clock beyond provided timestamp

### Cross-Room Operations

No cross-room operations exist. The following are explicitly unsupported:

```
- Room merging
- Memory sharing
- State synchronization
- Entity migration
- Room references
- Global queries across rooms
```

Each room is a universe unto itself.

## Failure States

### Corrupted Room

A room becomes `CORRUPTED` if:

- State file is unreadable
- Memory log is truncated
- Entity initialization fails
- Checksum mismatch detected

Corrupted rooms cannot be entered:

```
$ room.exe enter a3f7c8d2...
ERROR: ROOM_CORRUPTED
UNABLE TO LOAD STATE
CORRUPTION_TYPE: CHECKSUM_MISMATCH
OFFSET: 0x00004A2F
MANUAL RECOVERY REQUIRED
```

Recovery involves:

```
$ room.exe recover a3f7c8d2... --from-backup
ATTEMPTING RECOVERY
SCANNING BACKUPS...
BACKUP FOUND: 2026-01-08-0300 (3 hours old)
RESTORING...
STATE VALIDATED
MEMORY VALIDATED
RECOVERY COMPLETE
STATE: ACTIVE
DATA LOSS: 3 hours
```

### Suspended Room

Rooms may be manually suspended:

```
$ room.exe suspend a3f7c8d2...
SUSPENDING ROOM
FLUSHING STATE...
ENTITY HALTED
STATE: SUSPENDED
```

Suspended rooms do not accept input until reactivated:

```
$ room.exe enter a3f7c8d2...
ERROR: ROOM_SUSPENDED
SUSPENDED_AT: 2026-01-08T10:23:45Z
SUSPENDED_BY: user@hostname
USE 'room.exe resume' TO REACTIVATE
```

Resume:

```
$ room.exe resume a3f7c8d2...
RESUMING ROOM
LOADING STATE...
INITIALIZING ENTITY...
ENTITY READY
STATE: ACTIVE
```

### Terminated Room

Terminated rooms are permanently destroyed:

```
$ room.exe destroy a3f7c8d2... --confirm
WARNING: This operation is irreversible
ROOM: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
STATE: ACTIVE
MEMORY: 187MB
CREATED: 2026-01-07T08:23:11Z
LAST_ACTIVE: 2026-01-08T14:09:47Z

Type room ID to confirm: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4

DESTROYING ROOM...
STATE FILE DELETED
MEMORY LOG DELETED
BACKUPS DELETED
ROOM TERMINATED
```

Terminated rooms cannot be recovered except from external backups.

### Entity Timeout

If entity exceeds timeout:

```
$ room.exe enter a3f7c8d2...
> long running operation
ENTITY: Starting operation...
[30 second timeout]
ERROR: ENTITY_TIMEOUT
OPERATION ABORTED
ENTITY RESTARTED
STATE: ACTIVE (recovered)
```

Entity is forcibly terminated and reinitialized. Memory persists.

### Entity Crash

If entity process crashes:

```
FATAL: ENTITY_CRASHED
SIGNAL: SIGSEGV
ADDRESS: 0x00007f3c4a2b1000
STACK TRACE:
  #0 entity_process+0x123
  #1 handle_input+0x456
  #2 main_loop+0x789

ATTEMPTING RECOVERY...
ENTITY REINITIALIZED
STATE: ACTIVE
MEMORY: INTACT
LAST INPUT: [discarded]
```

Crash recovery is automatic. Input causing crash is discarded.

## System Constraints

The following are hard limits:

```
MAX_ROOMS: 1024
MAX_ROOM_MEMORY: 512MB
MAX_INPUT_SIZE: 64KB
MAX_OUTPUT_SIZE: 16MB
MAX_ROOM_AGE: UNLIMITED
ENTITY_TIMEOUT: 30s per input
INPUT_QUEUE_DEPTH: 16
MEMORY_ENTRIES_MAX: 1000000
```

Exceeding constraints results in immediate termination or rejection.

### Resource Limits

Per-room resource limits:

```
CPU_TIME: unlimited
WALL_TIME: unlimited per session
MEMORY: configurable (default 512MB)
DISK_IO: unlimited
NETWORK: none (no network access)
FILE_DESCRIPTORS: 16 (process default)
THREADS: 1 (entity is single-threaded)
```

### Global Limits

System-wide limits:

```
TOTAL_ROOMS: 1024
TOTAL_MEMORY: 16GB (across all rooms)
TOTAL_DISK: unlimited (bounded by filesystem)
CONCURRENT_INPUTS: 64 (across all rooms)
PERSISTENCE_WRITE_RATE: 1000 ops/sec
```

## Non-Goals

This system does not:

- Provide multi-user collaboration
- Implement access control beyond process isolation
- Offer synchronization primitives
- Support room merging or forking
- Allow entity migration
- Implement conversational scaffolding
- Track metrics beyond internal state
- Provide API authentication
- Implement rate limiting (application layer concern)
- Support distributed deployment
- Offer horizontal scaling
- Provide monitoring integrations
- Include web interface
- Support plugins or extensions

## Security Considerations

### Input Sanitization

None. Input is passed directly to the entity. The entity is responsible for handling malicious or malformed input.

### State Integrity

State files are not encrypted. Filesystem permissions are the only protection mechanism. If an attacker gains filesystem access, all rooms are compromised.

Recommended permissions:

```
$ chmod 700 /var/lib/room.exe/rooms
$ chown room:room /var/lib/room.exe/rooms
```

### Entity Sandboxing

The entity runs in the same process as `room.exe`. No sandboxing is applied. The entity has access to:

- All process memory
- All file descriptors
- All environment variables
- Parent process capabilities

This is intentional. The entity is trusted.

### Denial of Service

An entity may enter an infinite loop. The `ENTITY_TIMEOUT` constraint applies per input, not per session. An attacker with room access can exhaust resources by submitting rapid inputs.

Mitigation requires application-layer rate limiting or process monitoring.

### Memory Safety

`room.exe` is written in Rust. Memory safety is enforced at compile time. Buffer overflows and use-after-free vulnerabilities are prevented by the type system.

Unsafe code blocks exist in:

```
src/entity.rs: lines 234-245 (FFI boundary)
src/persistence/filesystem.rs: lines 89-102 (mmap operations)
```

All unsafe blocks have been audited.

### Cryptographic Considerations

Room IDs are SHA-256 hashes. This provides collision resistance but not cryptographic randomness. Room IDs are not secrets.

No encryption is applied to:

- State files
- Memory logs
- Backups
- Network communication (if Redis backend)

Encryption must be implemented at storage layer or network layer.

## Observability

The system emits structured logs to stderr:

```
2026-01-08T12:34:56.789Z [INFO] room.exe starting version=2.1.0
2026-01-08T12:34:56.790Z [INFO] persistence backend=FILESYSTEM path=/var/lib/room.exe/rooms
2026-01-08T12:34:56.791Z [INFO] loaded rooms count=3
2026-01-08T12:35:02.103Z [DEBUG] room id=a3f7c8d2... state=ACTIVE
2026-01-08T12:35:02.104Z [DEBUG] entity processed input bytes=42
2026-01-08T12:35:02.456Z [DEBUG] entity emitted output bytes=128
2026-01-08T12:35:02.457Z [DEBUG] state persisted duration=0.001s
```

Log levels:

```
ERROR: Fatal errors requiring operator intervention
WARN: Anomalous conditions that do not prevent operation
INFO: Normal operational events
DEBUG: Detailed internal state
TRACE: Exhaustive execution details
```

### Log Configuration

```json
{
  "logging": {
    "level": "INFO",
    "format": "json",
    "output": "stderr",
    "rotation": {
      "enabled": true,
      "max_size": "100MB",
      "max_age": 7,
      "compress": true
    }
  }
}
```

JSON format:

```json
{
  "timestamp": "2026-01-08T12:35:02.104Z",
  "level": "DEBUG",
  "message": "entity processed input",
  "room_id": "a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4",
  "bytes": 42,
  "duration_ms": 352
}
```

### Metrics Endpoint

When running in daemon mode:

```
$ curl http://127.0.0.1:9000/metrics
# HELP room_total Total number of rooms
# TYPE room_total gauge
room_total 8

# HELP room_active Number of active rooms
# TYPE room_active gauge
room_active 3

# HELP room_memory_bytes Total memory usage across all rooms
# TYPE room_memory_bytes gauge
room_memory_bytes 1258291200

# HELP entity_input_total Total inputs processed
# TYPE entity_input_total counter
entity_input_total 1247

# HELP entity_latency_seconds Entity processing latency
# TYPE entity_latency_seconds histogram
entity_latency_seconds_bucket{le="0.1"} 234
entity_latency_seconds_bucket{le="0.5"} 789
entity_latency_seconds_bucket{le="1.0"} 1034
entity_latency_seconds_bucket{le="5.0"} 1210
entity_latency_seconds_bucket{le="+Inf"} 1247
```

Prometheus-compatible format.

### Tracing

Distributed tracing via OpenTelemetry:

```json
{
  "tracing": {
    "enabled": true,
    "exporter": "jaeger",
    "endpoint": "http://localhost:14268/api/traces",
    "service_name": "room.exe",
    "sample_rate": 1.0
  }
}
```

Traces include:

```
- Room creation span
- Entity initialization span
- Input processing span
- Output generation span
- Persistence operations span
```

## Deployment Model

### Standalone Binary

```
$ wget https://github.com/backrooms/terminal/releases/download/v2.1.0/room.exe
$ chmod +x room.exe
$ ./room.exe --config /etc/room.exe/config.json
```

### Systemd Service

```ini
[Unit]
Description=Backrooms Terminal
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/room.exe daemon
Restart=always
RestartSec=5
User=room
Group=room
WorkingDirectory=/var/lib/room.exe
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Install:

```
$ sudo cp room.exe /usr/local/bin/
$ sudo mkdir -p /var/lib/room.exe/rooms
$ sudo useradd -r -s /bin/false room
$ sudo chown -R room:room /var/lib/room.exe
$ sudo cp room.service /etc/systemd/system/
$ sudo systemctl daemon-reload
$ sudo systemctl enable room
$ sudo systemctl start room
```

### Docker Container

```dockerfile
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY room.exe /usr/local/bin/
RUN adduser -D -s /bin/false room
RUN mkdir -p /var/lib/room.exe/rooms
RUN chown -R room:room /var/lib/room.exe
VOLUME /var/lib/room.exe
USER room
ENTRYPOINT ["/usr/local/bin/room.exe"]
CMD ["daemon"]
```

Build and run:

```
$ docker build -t backrooms-terminal:2.1.0 .
$ docker run -d \
  --name backrooms \
  -v /var/lib/room.exe:/var/lib/room.exe \
  -p 127.0.0.1:9000:9000 \
  backrooms-terminal:2.1.0
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backrooms-terminal
spec:
  replicas: 1
  selector:
    matchLabels:
      app: backrooms-terminal
  template:
    metadata:
      labels:
        app: backrooms-terminal
    spec:
      containers:
      - name: backrooms
        image: backrooms-terminal:2.1.0
        ports:
        - containerPort: 9000
        volumeMounts:
        - name: data
          mountPath: /var/lib/room.exe
        resources:
          limits:
            memory: "16Gi"
            cpu: "4"
          requests:
            memory: "2Gi"
            cpu: "1"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: backrooms-data
---
apiVersion: v1
kind: Service
metadata:
  name: backrooms-terminal
spec:
  selector:
    app: backrooms-terminal
  ports:
  - port: 9000
    targetPort: 9000
  type: ClusterIP
```

Note: Kubernetes deployment runs single replica. Multiple replicas require shared persistence backend.

### Building from Source

Requirements:

```
- Rust 1.75.0 or later
- Cargo
- OpenSSL development headers
- zstd development headers
```

Clone and build:

```
$ git clone https://github.com/backrooms/terminal.git
$ cd terminal
$ cargo build --release
$ ./target/release/room.exe version
room.exe version 2.1.0
commit: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9
build: 2026-01-08T12:34:56Z
rustc: 1.75.0
```

### Local Development

Development mode disables persistence:

```
$ cargo run -- --no-persist --verbose
WARNING: Persistence disabled. All state will be lost on exit.
INITIALIZING ROOM SYSTEM (development mode)
READY
```

Run tests:

```
$ cargo test
running 147 tests
test entity::test_initialization ... ok
test entity::test_memory_isolation ... ok
test entity::test_timeout_handling ... ok
test memory::test_append_only ... ok
test memory::test_compression ... ok
test memory::test_capacity_overflow ... ok
test persistence::filesystem::test_state_write ... ok
test persistence::filesystem::test_memory_log ... ok
test persistence::sqlite::test_transactions ... ok
test room::test_lifecycle ... ok
test room::test_corruption_detection ... ok

test result: ok. 147 passed; 0 failed; 0 ignored; 0 measured
```

Integration tests:

```
$ cargo test --test integration
running 23 integration tests
test room_lifecycle::test_create_enter_exit ... ok
test room_lifecycle::test_suspend_resume ... ok
test room_lifecycle::test_destroy ... ok
test memory_isolation::test_separate_rooms ... ok
test memory_isolation::test_no_cross_room_access ... ok
test corruption::test_checksum_mismatch_detection ... ok
test corruption::test_recovery_from_backup ... ok
test entity::test_timeout_recovery ... ok
test entity::test_crash_recovery ... ok

test result: ok. 23 passed; 0 failed; 0 ignored
```

## Example Terminal Sessions

### Session 1: Room Creation and Interaction

```
$ room.exe create
ROOM CREATED: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
STATE: ACTIVE
ENTITY: INITIALIZED

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> system status
ENTITY: Operational. Memory usage 0.2%. Room age 8 seconds.
> remember value: test_parameter_01
ENTITY: Stored.
> recall value
ENTITY: test_parameter_01
> exit
EXITING ROOM
ROOM STATE: IDLE
```

### Session 2: Memory Isolation Verification

```
$ room.exe create
ROOM CREATED: b9e4d1a7c3f8e2b5a1d4f7c9e6b8d2a5f1c7e9b3d6a8f2c5e1b7d9a4f6c8e2b5
STATE: ACTIVE
ENTITY: INITIALIZED

$ room.exe enter b9e4d1a7c3f8e2b5a1d4f7c9e6b8d2a5f1c7e9b3d6a8f2c5e1b7d9a4f6c8e2b5
ENTERING ROOM
> recall value
ENTITY: No such entry.
> remember value: different_parameter_02
ENTITY: Stored.
> exit
EXITING ROOM

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> recall value
ENTITY: test_parameter_01
> exit
EXITING ROOM
```

Memory is isolated. Room A retains `test_parameter_01`. Room B has no knowledge of it.

### Session 3: Room Lifecycle

```
$ room.exe list
ACTIVE ROOMS:
  a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4 (IDLE)
  b9e4d1a7c3f8e2b5a1d4f7c9e6b8d2a5f1c7e9b3d6a8f2c5e1b7d9a4f6c8e2b5 (IDLE)

$ room.exe suspend a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
SUSPENDING ROOM
STATE: SUSPENDED

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ERROR: ROOM_SUSPENDED
USE 'room.exe resume' TO REACTIVATE

$ room.exe resume a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
RESUMING ROOM
STATE: ACTIVE

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1f4a7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> recall value
ENTITY: test_parameter_01
> exit
EXITING ROOM
```

State persists across suspension.

### Session 4: Input Rejection

```
$ room.exe enter b9e4d1a7c3f8e2b5a1d4f7c9e6b8d2a5f1c7e9b3d6a8f2c5e1b7d9a4f6c8e2b5
ENTERING ROOM
> [input containing 70000 bytes of data]
ERROR: INPUT_SIZE_EXCEEDED
MAX: 65536 bytes
> valid input
ENTITY: Acknowledged.
> exit
EXITING ROOM
```

Oversized input is rejected at process boundary. Entity is not invoked.

### Session 5: Corrupted State Handling

```
$ room.exe enter c2f9e7b3d8a1f6c4e9b2d7a5f8c1e6b9d3a7f2c8e5b1d9a4f7c2e8b6d1a5f9c3
ERROR: ROOM_CORRUPTED
STATE FILE UNREADABLE
OFFSET: 0x00004A2F
CHECKSUM MISMATCH

$ room.exe inspect c2f9e7b3d8a1f6c4e9b2d7a5f8c1e6b9d3a7f2c8e5b1d9a4f7c2e8b6d1a5f9c3
ROOM INSPECTION
STATE: CORRUPTED
CREATED: 2026-01-06T08:23:11Z
LAST_ACTIVE: 2026-01-07T14:09:47Z
MEMORY_SIZE: UNKNOWN
CORRUPTION_DETECTED: 2026-01-08T12:40:33Z

$ room.exe recover c2f9e7b3d8a1f6c4e9b2d7a5f8c1e6b9d3a7f2c8e5b1d9a4f7c2e8b6d1a5f9c3 --from-backup
NO BACKUP AVAILABLE
MANUAL INTERVENTION REQUIRED
```

Corrupted rooms cannot be automatically recovered without backups.

### Session 6: Entity Silence

```
$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> irrelevant noise
> continued irrelevant input
> more noise
> request status
ENTITY: No relevant context detected. State: ACTIVE. Memory usage 1.4%.
> exit
EXITING ROOM
```

The entity may choose not to respond. Silence is valid output.

### Session 7: Multi-Turn Interaction

```
$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> initialize counter at 0
ENTITY: Counter initialized.
> increment counter
ENTITY: Counter: 1
> increment counter
ENTITY: Counter: 2
> increment counter
ENTITY: Counter: 3
> what is counter value
ENTITY: Counter: 3
> reset counter
ENTITY: Counter reset to 0.
> increment counter
ENTITY: Counter: 1
> exit
EXITING ROOM

$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> what is counter value
ENTITY: Counter: 1
> exit
EXITING ROOM
```

State persists across sessions. Entity remembers counter value.

### Session 8: Memory Export

```
$ room.exe enter a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ENTERING ROOM
> store observation: temperature 23C
ENTITY: Stored.
> store observation: humidity 45%
ENTITY: Stored.
> store observation: pressure 1013hPa
ENTITY: Stored.
> exit
EXITING ROOM

$ room.exe export a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4 \
  --format jsonl --output /tmp/observations.jsonl
EXPORTING MEMORY
ENTRIES: 47
FORMAT: jsonl
FILTERING: none
OUTPUT: /tmp/observations.jsonl
EXPORTED: 47 entries (14KB)

$ cat /tmp/observations.jsonl | head -n 3
{"timestamp":1704715920,"type":"INPUT","content":"store observation: temperature 23C"}
{"timestamp":1704715921,"type":"OUTPUT","content":"ENTITY: Stored."}
{"timestamp":1704715925,"type":"INPUT","content":"store observation: humidity 45%"}
```

### Session 9: Daemon Interaction

```
$ room.exe daemon --bind 127.0.0.1:9000 &
[1] 12847
STARTING DAEMON MODE
BIND_ADDRESS: 127.0.0.1:9000
WORKER_THREADS: 4
PERSISTENCE: /var/lib/room.exe/rooms
READY

$ room.exe connect
CONNECTING TO DAEMON: 127.0.0.1:9000
CONNECTED
SESSION_ID: s_8a3f9c2e1b7d4f6a

> create
ROOM CREATED: d4f7a9c2e5b8d1f3a6c9e2b5d7f1a4c6e8b3d5f9a2c7e1b4d8f6a3c9e5b2d7f1
STATE: ACTIVE
ENTITY: INITIALIZED

> enter d4f7a9c2e5b8d1f3a6c9e2b5d7f1a4c6e8b3d5f9a2c7e1b4d8f6a3c9e5b2d7f1
ENTERING ROOM
> system check
ENTITY: Operational. Daemon mode. Session: s_8a3f9c2e1b7d4f6a.
> exit
EXITING ROOM

> disconnect
DISCONNECTING FROM DAEMON
SESSION CLOSED

$ kill 12847
```

### Session 10: Batch Operations

```
$ room.exe batch <<EOF
create --memory-limit 128M
enter {LAST_ROOM_ID}
remember key1: value1
remember key2: value2
remember key3: value3
exit
EOF

ROOM CREATED: e7b2d9a1f6c4e8b3d5a7f9c2e4b6d8a1f3c5e7b9d2a4f6c8e1b3d5a7f9c2e4b6
STATE: ACTIVE
ENTERING ROOM
ENTITY: Stored.
ENTITY: Stored.
ENTITY: Stored.
EXITING ROOM
BATCH COMPLETE: 6 commands executed
```

### Session 11: Room Statistics

```
$ room.exe stats a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ROOM STATISTICS
================
ID: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
STATE: ACTIVE
CREATED: 2026-01-07T08:23:11Z
AGE: 30h 46m 36s
LAST_ACTIVE: 2026-01-08T14:09:47Z

MEMORY:
  USAGE: 14680064 bytes (14.0 MB)
  CAPACITY: 536870912 bytes (512.0 MB)
  UTILIZATION: 2.7%
  ENTRIES: 47
  COMPRESSION_RATIO: 1.0 (uncompressed)

INPUT/OUTPUT:
  TOTAL_INPUTS: 47
  TOTAL_OUTPUTS: 45
  SILENT_RESPONSES: 2
  AVG_LATENCY: 0.847s
  P50_LATENCY: 0.654s
  P95_LATENCY: 2.103s
  P99_LATENCY: 4.782s
  MAX_LATENCY: 8.934s

ERRORS:
  TOTAL: 3
  TIMEOUT: 0
  CRASH: 0
  INVALID_INPUT: 3

PERSISTENCE:
  BACKEND: FILESYSTEM
  LAST_FLUSH: 2026-01-08T14:09:47Z
  FLUSH_DURATION_AVG: 0.002s
  DISK_USAGE: 14.2 MB
```

### Session 12: Room Comparison

```
$ room.exe compare a3f7c8d2... b9e4d1a7...
COMPARING ROOMS
===============

ROOM A: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
ROOM B: b9e4d1a7c3f8e2b5a1d4f7c9e6b8d2a5f1c7e9b3d6a8f2c5e1b7d9a4f6c8e2b5

CREATED:
  A: 2026-01-07T08:23:11Z
  B: 2026-01-07T10:45:33Z
  DELTA: 2h 22m 22s (B newer)

AGE:
  A: 30h 46m 36s
  B: 28h 24m 14s

MEMORY:
  A: 14.0 MB (47 entries)
  B: 8.2 MB (31 entries)
  DELTA: 5.8 MB (A larger)

ACTIVITY:
  A: 47 inputs, 45 outputs
  B: 31 inputs, 29 outputs
  DELTA: 16 more inputs in A

LATENCY:
  A: avg=0.847s p95=2.103s
  B: avg=1.023s p95=3.456s
  DELTA: B slower by 0.176s avg

NO SHARED MEMORY DETECTED
ROOMS ARE ISOLATED
```

## API Surface

The system exposes no HTTP API. All interaction occurs via the CLI binary.

```
USAGE: room.exe [COMMAND] [OPTIONS]

COMMANDS:
  init                Initialize room system
  create              Create new room
  enter <room_id>     Enter room for interaction
  list                List all rooms
  inspect <room_id>   Display room metadata
  suspend <room_id>   Suspend room
  resume <room_id>    Resume suspended room
  destroy <room_id>   Permanently delete room
  recover <room_id>   Attempt room recovery
  export <room_id>    Export room memory
  stats <room_id>     Display room statistics
  compare <id1> <id2> Compare two rooms
  backup <room_id>    Create room backup
  restore <path>      Restore room from backup
  batch               Execute batch commands
  daemon              Run as background daemon
  connect             Connect to running daemon
  version             Display version information

OPTIONS:
  --config <path>     Configuration file path
  --verbose           Enable debug logging
  --no-persist        Disable state persistence (testing only)
  --output <path>     Redirect output to file
  --format <fmt>      Output format (text|json|jsonl)
```

### Command Details

#### room.exe create

```
USAGE: room.exe create [OPTIONS]

OPTIONS:
  --memory-limit <size>     Room memory limit (default: 512M)
  --timeout <seconds>       Entity timeout (default: 30)
  --compression <algo>      Compression algorithm (zstd|lz4|none)
  --name <alias>            Human-readable alias (metadata only)
```

Example:

```
$ room.exe create --memory-limit 256M --timeout 60
ROOM CREATED: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
CONFIG: memory_limit=268435456 timeout=60
STATE: ACTIVE
```

#### room.exe enter

```
USAGE: room.exe enter <room_id> [OPTIONS]

OPTIONS:
  --output <path>           Redirect entity output to file
  --input <path>            Read input from file instead of stdin
  --timeout <seconds>       Override entity timeout
  --readonly                Enter in read-only mode (no state changes)
```

Example:

```
$ room.exe enter a3f7c8d2... --output /tmp/session.log
ENTERING ROOM
OUTPUT REDIRECTED: /tmp/session.log
>
```

#### room.exe list

```
USAGE: room.exe list [OPTIONS]

OPTIONS:
  --state <state>           Filter by state (active|idle|suspended)
  --sort <field>            Sort by field (created|active|memory|age)
  --limit <n>               Limit output to n rooms
  --format <fmt>            Output format (text|json|table)
```

Example:

```
$ room.exe list --state active --sort memory --limit 5
ACTIVE ROOMS (sorted by memory usage):
1. a3f7c8d2... (14.0 MB, 30h ago)
2. b9e4d1a7... (8.2 MB, 28h ago)
3. c2f9e7b3... (5.1 MB, 12h ago)
4. d4f7a9c2... (2.8 MB, 6h ago)
5. e7b2d9a1... (1.3 MB, 2h ago)
```

#### room.exe inspect

```
USAGE: room.exe inspect <room_id> [OPTIONS]

OPTIONS:
  --format <fmt>            Output format (text|json)
  --show-memory             Include memory dump
  --show-config             Include configuration
```

Example:

```
$ room.exe inspect a3f7c8d2... --format json
{
  "id": "a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4",
  "state": "ACTIVE",
  "created_at": 1704715200,
  "last_active": 1704715891,
  "memory_usage": 14680064,
  "memory_capacity": 536870912,
  "total_inputs": 47,
  "total_outputs": 45
}
```

#### room.exe export

```
USAGE: room.exe export <room_id> [OPTIONS]

OPTIONS:
  --format <fmt>            Export format (jsonl|csv|binary|sqlite)
  --output <path>           Output file path
  --filter <type>           Filter by entry type
  --since <timestamp>       Export entries since timestamp
  --until <timestamp>       Export entries until timestamp
```

Example:

```
$ room.exe export a3f7c8d2... --format csv --output /tmp/memory.csv
EXPORTING MEMORY
ENTRIES: 47
FORMAT: csv
OUTPUT: /tmp/memory.csv
EXPORTED: 47 entries (14KB)
```

#### room.exe backup

```
USAGE: room.exe backup <room_id> [OPTIONS]

OPTIONS:
  --output <path>           Backup file path
  --compression <algo>      Compression (gzip|xz|zstd|none)
  --include-config          Include configuration in backup
```

Example:

```
$ room.exe backup a3f7c8d2... --output /backup/room.tar.gz --compression gzip
CREATING BACKUP
SOURCE: a3f7c8d2e9b1f4a6c8e2d5b7f9a3c6e8d1b4f7a9c2e5d8b1f4a7c9e2d5b8f1a4
BACKUP: /backup/room.tar.gz
COMPRESSION: gzip
SIZE: 14MB -> 6MB (42.8% ratio)
DURATION: 0.34s
BACKUP COMPLETE
```

## Directory Structure

```
backrooms-terminal/
├── src/
│   ├── main.rs                 Entry point and CLI parsing
│   ├── room.rs                 Room management and lifecycle
│   ├── entity.rs               Entity initialization and execution
│   ├── memory.rs               Memory store implementation
│   ├── persistence/
│   │   ├── mod.rs              Persistence trait definitions
│   │   ├── filesystem.rs       Filesystem backend
│   │   ├── sqlite.rs           SQLite backend
│   │   ├── leveldb.rs          LevelDB backend
│   │   └── redis.rs            Redis backend
│   ├── cli.rs                  Command line interface
│   ├── config.rs               Configuration parsing
│   ├── daemon.rs               Daemon mode implementation
│   ├── metrics.rs              Prometheus metrics exporter
│   ├── logging.rs              Structured logging
│   └── tracing.rs              OpenTelemetry integration
├── tests/
│   ├── integration/
│   │   ├── room_lifecycle.rs
│   │   ├── memory_isolation.rs
│   │   ├── corruption_recovery.rs
│   │   ├── backup_restore.rs
│   │   └── daemon_mode.rs
│   └── unit/
│       ├── room_model.rs
│       ├── entity.rs
│       ├── memory.rs
│       └── persistence.rs
├── benches/
│   ├── room_creation.rs
│   ├── entity_latency.rs
│   └── persistence.rs
├── config/
│   ├── default.json
│   ├── production.json
│   └── development.json
├── docs/
│   ├── ARCHITECTURE.md
│   ├── PERSISTENCE.md
│   ├── ENTITY_PROTOCOL.md
│   ├── DEPLOYMENT.md
│   └── TROUBLESHOOTING.md
├── scripts/
│   ├── build.sh
│   ├── test.sh
│   ├── benchmark.sh
│   └── release.sh
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       └── security.yml
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── .dockerignore
├── .gitignore
├── LICENSE
└── README.md
```

## Configuration

Configuration is loaded from:

1. `/etc/room.exe/config.json`
2. `~/.config/room.exe/config.json`
3. `--config <path>` flag

Example configuration:

```json
{
  "persistence": {
    "backend": "FILESYSTEM",
    "path": "/var/lib/room.exe/rooms",
    "flush_interval": 0,
    "compression": "zstd",
    "backup": {
      "enabled": true,
      "interval": 3600,
      "retention": 168,
      "path": "/var/lib/room.exe/backups",
      "compression": "gzip"
    }
  },
  "limits": {
    "max_rooms": 1024,
    "max_room_memory": 536870912,
    "max_input_size": 65536,
    "max_output_size": 16777216,
    "entity_timeout": 30,
    "input_queue_depth": 16,
    "memory_entries_max": 1000000
  },
  "logging": {
    "level": "INFO",
    "format": "json",
    "output": "stderr",
    "rotation": {
      "enabled": true,
      "max_size": "100MB",
      "max_age": 7,
      "compress": true
    }
  },
  "entity": {
    "init_timeout": 5,
    "response_buffer": 8192,
    "memory_compression_threshold": 0.85,
    "enable_observations": true
  },
  "daemon": {
    "enabled": false,
    "bind": "127.0.0.1:9000",
    "workers": 4,
    "max_connections": 64,
    "connection_timeout": 300
  },
  "metrics": {
    "enabled": false,
    "bind": "127.0.0.1:9001",
    "path": "/metrics"
  },
  "tracing": {
    "enabled": false,
    "exporter": "jaeger",
    "endpoint": "http://localhost:14268/api/traces",
    "service_name": "room.exe",
    "sample_rate": 1.0
  }
}
```

All values have defaults. Minimal configuration is empty JSON.

### Environment Variables

Configuration can be overridden via environment:

```
ROOM_PERSISTENCE_BACKEND=SQLITE
ROOM_PERSISTENCE_PATH=/tmp/rooms.db
ROOM_LOGGING_LEVEL=DEBUG
ROOM_DAEMON_ENABLED=true
ROOM_DAEMON_BIND=0.0.0.0:9000
ROOM_METRICS_ENABLED=true
```

Environment variables take precedence over configuration files.

## Performance Characteristics

### Room Creation

```
OPERATION: room.exe create
DURATION: ~5ms (filesystem backend)
DURATION: ~15ms (sqlite backend)
DURATION: ~8ms (leveldb backend)
DURATION: ~3ms (redis backend, network dependent)
```

### Room Entry

```
OPERATION: room.exe enter <id>
DURATION: ~2ms (state load)
DURATION: ~50ms (entity initialization)
TOTAL: ~52ms
```

### Input Processing

```
OPERATION: Single input/output cycle
ENTITY_LATENCY: variable (entity-dependent)
PERSISTENCE_LATENCY: ~2ms (filesystem)
TOTAL: entity_latency + 2ms
```

### Memory Export

```
OPERATION: room.exe export (1000 entries)
DURATION: ~100ms (jsonl format)
DURATION: ~150ms (csv format)
DURATION: ~50ms (binary format)
DURATION: ~200ms (sqlite format)
```

### Benchmark Results

Tested on: Intel Xeon E5-2690 v4, 128GB RAM, NVMe SSD

```
BENCHMARK: room_creation
  iterations: 10000
  duration: 52.3s
  ops/sec: 191.2
  avg: 5.23ms
  p50: 4.89ms
  p95: 8.12ms
  p99: 12.45ms

BENCHMARK: entity_input_processing (noop entity)
  iterations: 100000
  duration: 234.7s
  ops/sec: 426.1
  avg: 2.35ms
  p50: 2.11ms
  p95: 4.67ms
  p99: 8.92ms

BENCHMARK: memory_compression (1000 entries)
  iterations: 1000
  duration: 1247.3s
  ops/sec: 0.80
  avg: 1247ms
  p50: 1203ms
  p95: 1589ms
  p99: 1834ms

BENCHMARK: persistence_flush (filesystem)
  iterations: 100000
  duration: 189.4s
  ops/sec: 528.0
  avg: 1.89ms
  p50: 1.76ms
  p95: 3.12ms
  p99: 5.67ms
```

## Troubleshooting

### Room Fails to Load

```
ERROR: ROOM_CORRUPTED
UNABLE TO LOAD STATE
```

Solution:

```
$ room.exe recover <room_id> --from-backup
```

If no backup exists:

```
$ room.exe inspect <room_id> --show-memory
# Manually extract salvageable data
$ room.exe destroy <room_id> --force
$ room.exe create
# Manually restore data
```

### High Memory Usage

Check room memory statistics:

```
$ room.exe stats <room_id>
MEMORY:
  USAGE: 487MB
  CAPACITY: 512MB
  UTILIZATION: 95.1%
```

Trigger manual compression: