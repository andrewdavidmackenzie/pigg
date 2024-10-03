---
name: Release Manual testing template
about: A checklist of manual tests to do before a release
title: Release Manual Test Checklist
labels: ''
assignees: ''

---

## Manual Tests

- [ ] piggui can connect to piglet using iroh by entering nodeid in connect dialog
- [ ] piggui can connect to piglet using tcp by entering ip and port in connect dialog
- [ ] piggui runs on RPi and can control an output and display an input

## Test Blocks

- HW Interaction
- UI Interaction
- Networking

## Scenarios

- Hardware can be fake hw (a simulation) or real GPIO hardware on RPi
- Scenario can be piggui on it's own (UI and HW control) or piggui (UI) + piglet (HW Control)

## Already automated tests

- [X] piggui connects to piglet using Iroh by supplying nodeid at the command line
- [X] piggui connects to piglet using Tcp by supplying an IP and port number at the command line
