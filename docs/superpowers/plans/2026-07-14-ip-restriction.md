# IP Restriction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a silent IP-based restriction for LANChat specifically targeting the `6.101.90.*` subnet.

**Architecture:** Inject logic at the beginning of `run()` in `src-tauri/src/lib.rs` to fetch IPv4 addresses. If any starts with `6.101.90.` but does not equal `6.101.90.110`, `6.101.90.116`, or `6.101.90.136`, the app silently exits (`std::process::exit(0)`).

**Tech Stack:** Rust (Tauri Backend)

## Global Constraints

- If the application is launched on a machine with an IPv4 address starting with `6.101.90.`, the application MUST ONLY run if the address is exactly `6.101.90.110`, `6.101.90.116`, or `6.101.90.136`.
- If the machine has a `6.101.90.*` address but it is not one of the allowed three, the application must terminate immediately.
- The termination must be silent: no UI windows, no error dialogs, just process exit.
- If the machine does not have any `6.101.90.*` addresses (e.g. running on `192.168.1.*` or `10.0.0.*`), it should operate normally.
- If `local_ip_address::list_afinet_netifas()` fails, bypass the check (fail-open) so legitimate users aren't locked out.

---

### Task 1: Add IP check logic to lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `local_ip_address::list_afinet_netifas()`
- Produces: Process termination (`std::process::exit(0)`) if unauthorized.

- [ ] **Step 1: Write the minimal implementation**
Since this is a manual test without automated tests for the `run()` function entry point (which spins up Tauri), we will insert the code directly at the top of `run()`.

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // === IP Restriction Logic Start ===
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        let mut has_6_101_90 = false;
        let mut is_allowed = false;

        for (_name, ip) in interfaces {
            if let std::net::IpAddr::V4(ipv4) = ip {
                let ip_str = ipv4.to_string();
                if ip_str.starts_with("6.101.90.") {
                    has_6_101_90 = true;
                    if ip_str == "6.101.90.110" || ip_str == "6.101.90.116" || ip_str == "6.101.90.136" {
                        is_allowed = true;
                    }
                }
            }
        }

        if has_6_101_90 && !is_allowed {
            std::process::exit(0);
        }
    }
    // === IP Restriction Logic End ===

    tauri::Builder::default()
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add IP restriction for 6.101.90 subnet"
```
