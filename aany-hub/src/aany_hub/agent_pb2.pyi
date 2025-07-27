from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class LogEntry(_message.Message):
    __slots__ = ["agent_id", "timestamp", "level", "message", "component", "metadata"]
    class MetadataEntry(_message.Message):
        __slots__ = ["key", "value"]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    AGENT_ID_FIELD_NUMBER: _ClassVar[int]
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    COMPONENT_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    agent_id: str
    timestamp: str
    level: str
    message: str
    component: str
    metadata: _containers.ScalarMap[str, str]
    def __init__(self, agent_id: _Optional[str] = ..., timestamp: _Optional[str] = ..., level: _Optional[str] = ..., message: _Optional[str] = ..., component: _Optional[str] = ..., metadata: _Optional[_Mapping[str, str]] = ...) -> None: ...

class AgentInfo(_message.Message):
    __slots__ = ["agent_id", "version", "hostname", "capabilities"]
    class CapabilitiesEntry(_message.Message):
        __slots__ = ["key", "value"]
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    AGENT_ID_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    HOSTNAME_FIELD_NUMBER: _ClassVar[int]
    CAPABILITIES_FIELD_NUMBER: _ClassVar[int]
    agent_id: str
    version: str
    hostname: str
    capabilities: _containers.ScalarMap[str, str]
    def __init__(self, agent_id: _Optional[str] = ..., version: _Optional[str] = ..., hostname: _Optional[str] = ..., capabilities: _Optional[_Mapping[str, str]] = ...) -> None: ...

class RegistrationResponse(_message.Message):
    __slots__ = ["success", "message", "session_id"]
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    success: bool
    message: str
    session_id: str
    def __init__(self, success: bool = ..., message: _Optional[str] = ..., session_id: _Optional[str] = ...) -> None: ...

class LogResponse(_message.Message):
    __slots__ = ["success", "entries_received"]
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    ENTRIES_RECEIVED_FIELD_NUMBER: _ClassVar[int]
    success: bool
    entries_received: int
    def __init__(self, success: bool = ..., entries_received: _Optional[int] = ...) -> None: ...

class HeartbeatRequest(_message.Message):
    __slots__ = ["agent_id", "session_id"]
    AGENT_ID_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    agent_id: str
    session_id: str
    def __init__(self, agent_id: _Optional[str] = ..., session_id: _Optional[str] = ...) -> None: ...

class HeartbeatResponse(_message.Message):
    __slots__ = ["alive", "server_time"]
    ALIVE_FIELD_NUMBER: _ClassVar[int]
    SERVER_TIME_FIELD_NUMBER: _ClassVar[int]
    alive: bool
    server_time: str
    def __init__(self, alive: bool = ..., server_time: _Optional[str] = ...) -> None: ...
