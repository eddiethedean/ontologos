#!/usr/bin/env bash
# Process hygiene for HermiT burndown scripts — stale cleanup, exclusive lock, trap on exit.
# Source from hermit-burndown.sh and related scripts; do not execute directly.
set -euo pipefail

: "${BURNDOWN_GUARD_ROOT_PID:=$$}"
: "${BURNDOWN_GUARD_LOCK_DIR:=/tmp/ontologos-burndown-${USER:-unknown}.lock}"
: "${BURNDOWN_GUARD_ACTIVE:=0}"
: "${BURNDOWN_GUARD_LOCK_HELD:=0}"

# Match conformance scans, cargo test/build, and generated test binaries.
_BURNDOWN_MATCH_RE='cargo test.*ontologos-conformance|cargo build.*ontologos-conformance.*--bins|/parity_status |/wg_failures |/promote_catalog |/promote_wg |/audit_planned_backlog |/engine_failures |/dl_failures |/dl_ofn_pass_rate |hermit_generated|hermit_wg_generated|phase4_closure|wg_phase4_check'

burndown_is_burndown_process() {
  local cmd="$1"
  [[ "${cmd}" == *"cargo test"* && "${cmd}" == *"ontologos-conformance"* ]] && return 0
  [[ "${cmd}" == *"cargo build"* && "${cmd}" == *"ontologos-conformance"* && "${cmd}" == *"--bins"* ]] && return 0
  [[ "${cmd}" =~ /(parity_status|wg_failures|promote_catalog|promote_wg|audit_planned_backlog|engine_failures|dl_failures|dl_ofn_pass_rate)( |$) ]] && return 0
  [[ "${cmd}" == *hermit_generated* ]] && return 0
  [[ "${cmd}" == *hermit_wg_generated* ]] && return 0
  [[ "${cmd}" == *phase4_closure* ]] && return 0
  [[ "${cmd}" == *wg_phase4_check* ]] && return 0
  return 1
}

burndown_is_descendant() {
  local pid="$1" ancestor="$2"
  while [[ -n "${pid}" && "${pid}" -gt 1 ]]; do
    [[ "${pid}" -eq "${ancestor}" ]] && return 0
    pid="$(ps -o ppid= -p "${pid}" 2>/dev/null | tr -d ' ' || true)"
  done
  return 1
}

# PIDs for burndown-related processes outside the current shell tree.
burndown_stale_pids() {
  local pid cmd
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    pid="${line%% *}"
    cmd="${line#${pid} }"
    [[ "${pid}" =~ ^[0-9]+$ ]] || continue
    burndown_is_descendant "${pid}" "${BURNDOWN_GUARD_ROOT_PID}" && continue
    burndown_is_burndown_process "${cmd}" || continue
    echo "${pid}"
  done < <(pgrep -fl "${_BURNDOWN_MATCH_RE}" 2>/dev/null || true)
}

burndown_list_stale() {
  local pid
  local found=0
  while IFS= read -r pid; do
    [[ -z "${pid}" ]] && continue
    found=1
    ps -o pid=,etime=,command= -p "${pid}" 2>/dev/null | sed 's/^/  /' || echo "  ${pid} (exited)"
  done < <(burndown_stale_pids | sort -u)
  if (( found )); then
    return 0
  fi
  return 1
}

burndown_kill_pids() {
  local signal="$1"
  shift
  local pid
  for pid in "$@"; do
    [[ -n "${pid}" ]] || continue
    kill "-${signal}" "${pid}" 2>/dev/null || true
  done
}

# Terminate stale burndown processes (SIGTERM → SIGKILL).
burndown_cleanup_stale() {
  local -a pids=()
  local pid
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] && pids+=("${pid}")
  done < <(burndown_stale_pids | sort -u)
  ((${#pids[@]} == 0)) && return 0

  echo "burndown: stopping ${#pids[@]} stale process(es)"
  burndown_kill_pids TERM "${pids[@]}"
  sleep 1
  burndown_kill_pids KILL "${pids[@]}"
}

burndown_acquire_lock() {
  if ! mkdir "${BURNDOWN_GUARD_LOCK_DIR}" 2>/dev/null; then
    local holder
    holder="$(cat "${BURNDOWN_GUARD_LOCK_DIR}/pid" 2>/dev/null || echo unknown)"
    echo "error: another burndown job is running (pid ${holder}, lock ${BURNDOWN_GUARD_LOCK_DIR})" >&2
    echo "  bash benchmarks/scripts/hermit-burndown.sh cleanup" >&2
    exit 1
  fi
  echo "${BURNDOWN_GUARD_ROOT_PID}" >"${BURNDOWN_GUARD_LOCK_DIR}/pid"
  BURNDOWN_GUARD_LOCK_HELD=1
}

burndown_release_lock() {
  [[ "${BURNDOWN_GUARD_LOCK_HELD}" -eq 1 ]] || return 0
  rm -rf "${BURNDOWN_GUARD_LOCK_DIR}"
  BURNDOWN_GUARD_LOCK_HELD=0
}

burndown_trap_cleanup() {
  local pid
  # Stop background jobs started by this shell (e.g. interrupted cargo test).
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] && kill -TERM "${pid}" 2>/dev/null || true
  done < <(jobs -pr 2>/dev/null || true)
  sleep 0.2
  while IFS= read -r pid; do
    [[ -n "${pid}" ]] && kill -KILL "${pid}" 2>/dev/null || true
  done < <(jobs -pr 2>/dev/null || true)
  burndown_release_lock
}

# Call at start of heavy burndown steps (triage, promote, test, test-full).
burndown_guard_begin() {
  if [[ "${BURNDOWN_GUARD_ACTIVE}" -eq 1 ]]; then
    return 0
  fi
  BURNDOWN_GUARD_ACTIVE=1
  trap burndown_trap_cleanup EXIT INT TERM

  if burndown_list_stale; then
    echo "burndown: clearing stale processes before starting" >&2
    burndown_cleanup_stale
  fi

  burndown_acquire_lock
}

# Optional: end guard early (lock released on EXIT trap anyway).
burndown_guard_end() {
  burndown_trap_cleanup
  trap - EXIT INT TERM
  BURNDOWN_GUARD_ACTIVE=0
}

# Standalone cleanup (hermit-burndown.sh cleanup).
burndown_guard_cleanup_cmd() {
  echo "burndown process check"
  if burndown_list_stale; then
    burndown_cleanup_stale
    echo "burndown: stale processes cleared"
  else
    echo "burndown: no stale processes"
  fi
  if [[ -d "${BURNDOWN_GUARD_LOCK_DIR}" ]]; then
    local holder
    holder="$(cat "${BURNDOWN_GUARD_LOCK_DIR}/pid" 2>/dev/null || echo unknown)"
    echo "burndown: removing stale lock (pid ${holder})"
    rm -rf "${BURNDOWN_GUARD_LOCK_DIR}"
  fi
}
