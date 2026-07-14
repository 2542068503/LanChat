# LANChat IP Restriction Design

## Goal
Implement a silent IP-based restriction for the LANChat application specifically targeting the `6.101.90.*` subnet.

## Requirements
- If the application is launched on a machine with an IPv4 address starting with `6.101.90.`, the application MUST ONLY run if the address is exactly `6.101.90.110`, `6.101.90.116`, or `6.101.90.136`.
- If the machine has a `6.101.90.*` address but it is not one of the allowed three, the application must terminate immediately.
- The termination must be silent: no UI windows, no error dialogs, just process exit.
- If the machine does not have any `6.101.90.*` addresses (e.g. running on `192.168.1.*` or `10.0.0.*`), it should operate normally.

## Architecture & Implementation
- **Location**: The logic will be injected at the very beginning of the `pub fn run()` in `src-tauri/src/lib.rs`.
- **API**: We will use `local_ip_address::list_afinet_netifas()` to iterate over all active network interfaces.
- **Logic**:
  1. Retrieve all IPv4 addresses.
  2. Filter for those starting with `6.101.90.`.
  3. If the filtered list is not empty, check if any of the addresses match the allowed list `["6.101.90.110", "6.101.90.116", "6.101.90.136"]`.
  4. If there's no match among the restricted subnet IPs, call `std::process::exit(0)`.
- **Dependencies**: The `local_ip_address` crate is already present in the project.

## Error Handling
- If `local_ip_address::list_afinet_netifas()` fails (returns an `Err`), the application should gracefully bypass the check (fail-open) so legitimate users aren't locked out due to API failures, or fail-closed if security is paramount. Given the context, we will fail-open (allow usage) if we can't retrieve IPs, though this failure mode is rare.

## Testing
- Manual testing by temporarily mocking the returned IPs to ensure termination and continuation work as intended.
