#!/usr/bin/env bash

# Private fixed-Reachy lifecycle shared by the scenario and teleop workflows.
# Callers own the shell and decide how a client or observer failure is reported.

sim_stack_start() {
  local repo_root=$1
  local visualize=$2
  local name=$3

  sim_stack_rt_log=$(mktemp "/tmp/soma-${name}-rt.XXXXXX.log")
  sim_stack_runtime_log=$(mktemp "/tmp/soma-${name}-runtime.XXXXXX.log")
  sim_stack_observer_log=$(mktemp "/tmp/soma-${name}-observer.XXXXXX.log")
  sim_stack_rerun_log=$(mktemp "/tmp/soma-${name}-rerun.XXXXXX.log")
  sim_stack_snapshot_socket="/tmp/soma-${name}-observer.$$.sock"

  if $visualize; then
    "$repo_root/scripts/cargo-mujoco" build --quiet -p soma-runtime \
      --features sim-visualization --bin robot-rt --bin robot-runtime \
      --bin robot-sim-observer
  else
    "$repo_root/scripts/cargo-mujoco" build --quiet --bin robot-rt --bin robot-runtime
  fi

  export LD_LIBRARY_PATH="$repo_root/.mujoco/mujoco-3.9.0/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  "$repo_root/target/debug/robot-runtime" >"$sim_stack_runtime_log" 2>&1 &
  sim_stack_runtime_pid=$!

  if $visualize; then
    local rerun_version
    rerun_version=$(cd "$repo_root/python" && UV_DEFAULT_INDEX=https://pypi.org/simple \
      uv run --extra visualization rerun --version)
    if [[ $rerun_version != "rerun-cli 0.36.2 "* ]]; then
      echo "expected Rerun viewer 0.36.2, got: $rerun_version" >&2
      return 1
    fi
    sim_stack_rerun_port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
    (cd "$repo_root/python" && exec setsid env UV_DEFAULT_INDEX=https://pypi.org/simple \
      uv run --extra visualization rerun --bind 127.0.0.1 --port "$sim_stack_rerun_port" \
      --expect-data-soon) >"$sim_stack_rerun_log" 2>&1 &
    sim_stack_rerun_pid=$!
    for _ in $(seq 1 100); do
      (exec 3<>/dev/tcp/127.0.0.1/"$sim_stack_rerun_port") 2>/dev/null && break
      kill -0 "$sim_stack_rerun_pid" 2>/dev/null || { cat "$sim_stack_rerun_log" >&2; return 1; }
      sleep 0.1
    done
    "$repo_root/target/debug/robot-sim-observer" \
      --snapshot-socket "$sim_stack_snapshot_socket" \
      --rerun-endpoint "rerun+http://127.0.0.1:$sim_stack_rerun_port/proxy" \
      >"$sim_stack_observer_log" 2>&1 &
    sim_stack_observer_pid=$!
    for _ in $(seq 1 100); do
      [[ -S $sim_stack_snapshot_socket && -f $sim_stack_snapshot_socket.ready ]] && break
      kill -0 "$sim_stack_observer_pid" 2>/dev/null || { cat "$sim_stack_observer_log" >&2; return 1; }
      sleep 0.1
    done
    if [[ ! -S $sim_stack_snapshot_socket || ! -f $sim_stack_snapshot_socket.ready ]] || \
       ! kill -0 "$sim_stack_observer_pid" 2>/dev/null; then
      cat "$sim_stack_observer_log" >&2
      return 1
    fi
    "$repo_root/target/debug/robot-rt" --observe-socket "$sim_stack_snapshot_socket" \
      >"$sim_stack_rt_log" 2>&1 &
  else
    "$repo_root/target/debug/robot-rt" >"$sim_stack_rt_log" 2>&1 &
  fi
  sim_stack_rt_pid=$!

  for _ in $(seq 1 50); do
    if [[ -S /tmp/soma-robot-rt.sock && -S /tmp/soma-robot-runtime.sock ]] && \
       (exec 3<>/dev/tcp/127.0.0.1/7447) 2>/dev/null; then
      return 0
    fi
    sleep 0.1
  done
  if ! kill -0 "$sim_stack_rt_pid" "$sim_stack_runtime_pid" 2>/dev/null || \
     ! (exec 3<>/dev/tcp/127.0.0.1/7447) 2>/dev/null; then
    cat "$sim_stack_rt_log" "$sim_stack_runtime_log" >&2
    return 1
  fi
}

sim_stack_cleanup() {
  if [[ -n ${sim_stack_rerun_pid:-} ]]; then
    kill -TERM -- "-$sim_stack_rerun_pid" 2>/dev/null || true
  fi
  kill "${sim_stack_rt_pid:-}" "${sim_stack_runtime_pid:-}" \
    "${sim_stack_observer_pid:-}" "${sim_stack_rerun_pid:-}" 2>/dev/null || true
  wait "${sim_stack_rt_pid:-}" "${sim_stack_runtime_pid:-}" \
    "${sim_stack_observer_pid:-}" "${sim_stack_rerun_pid:-}" 2>/dev/null || true
  rm -f "${sim_stack_snapshot_socket:-}" "${sim_stack_snapshot_socket:-}.lock" \
    "${sim_stack_snapshot_socket:-}.ready" "${sim_stack_snapshot_socket:-}.ready.lock"
}
