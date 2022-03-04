![logo](https://assets.gitlab-static.net/uploads/-/system/group/avatar/10553166/logo_white.ico?width=64)

# BP3D logger

A flexible [Log](https://docs.rs/log/latest/log/trait.Log.html) implementation intended to be used with BP3D software.

## Main features
- Support for file logging.
- Support for stdout/stderr logging.
- Support for in memory logger.
- Easily switch on/off logging backends.
- Logging backends should not panic/abort/crash the software in any way.

### stdout/stderr
- Error messages are written to stderr instead of stdout (configurable).
- If write fails, message is ignored; no panic produced.

### file logging
- If write fails, an error is sent to other logging backends, if other backends fails, message is ignored; no panic produced.

### in memory
- Log is limited to a fixed number of messages (configurable).

