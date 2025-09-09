![logo](https://assets.gitlab-static.net/uploads/-/system/group/avatar/10553166/logo_white.ico?width=64)

# Debug core

The debug core contains Rust APIs used to debug BP3D software. This contains a set of tools optimized for use by BP3D based projects with soft real-time constraints.

## Features

- A logger system with trace disabled in release builds for improved performance.
- A simple profiler system which can efficiently measure the time spent in Rust code scope.
- A trace system designed to trace asynchronous and long-running operations. 
