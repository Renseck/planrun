# planrun-tui

The planrun TUI, `planrun tui [<plans-dir>]`: Plans, the plan view in Inspect and Run, the
decision dock, History, the resume preview, the command palette and help. The design is
`docs/tui-design.md`; its decisions are D84–D94 in `docs/decisions.md`; its parts are described
in `docs/architecture.md` §7.1.

## How it is tested (D93)

Everything a test can see runs in-process: `driver::Core` over ratatui's `TestBackend`, fed
scripted keys, with the real runner on a copy of the greeter fixture (`tests/support/fixture.rs`).
Snapshots are cell grids (`grid::grid`): each row's characters, then its style runs by token
name, so they read against the design canvas's artboards cell for cell.

```powershell
$env:PLANRUN_REQUIRE_SHELLS = '1'; cargo test -p planrun-tui
```

## Manual check list

What only a real terminal shows. Run it in Windows Terminal with Cascadia Mono, from a copy of
the greeter fixture or any repository with plans in `docs/plans` (or pass the directory), after
`cargo build -p planrun-cli`. Each item is a manual step, checked by hand.

1. **Start and quit.** `planrun tui` shows Plans; `q` quits and the terminal is as it was
   (prompt, scrollback, cursor). `planrun tui | Out-Host` and `planrun tui < $null` refuse with
   exit code 2.
2. **Glyphs** (§5). Every status, kind and chrome glyph is one cell wide and none is a box or
   a question mark; look hardest at `⚑` (U+2691) and the spinner `◐◓◑◒` while a script runs.
   With `$env:PLANRUN_ASCII = '1'` every glyph is its ASCII fallback. A glyph that fails here is
   replaced by its fallback in the table (§5).
3. **Colours** (§4). The header, footer and bands are bg1; the selected row bg2 with `▌` in
   blue; green only on verified things, amber only where planrun waits for you.
4. **Keys** (§7). F1–F4, `tab`/`shift+tab`, `j`/`k`/arrows, `enter`, `esc`, `R` and `shift+R`,
   `a`, `c`, `/`, `ctrl+k`, `ctrl+p`. A held arrow repeats; a held letter does not. In VS Code's
   terminal `ctrl+k` and `ctrl+p` reach the TUI only with
   `terminal.integrated.sendKeybindingsToShell` set (D94).
5. **A run.** `R` on a plan runs it; the navigator, CHANGES and the console follow the run; a
   question opens the dock over the console. A typed question takes the whole word and enter,
   and refuses a word it does not take.
6. **The `[o]` shell.** At a question that offers it, `o` leaves the screen and starts pwsh at
   the repository root; Ctrl-C in that shell reaches the shell (a running `ping -t` stops, the
   shell stays); `exit` comes back to the same question, the screen redrawn.
7. **Ctrl-C** (D91). During a run, Ctrl-C opens the END PLANRUN confirm; `n` keeps running; `y`
   restores the terminal and ends planrun mid-block, the session's processes gone (Task
   Manager); `planrun resume` (or History, F4) goes on from the journal.
8. **Resize.** Shrinking the window below 100 × 30 shows "terminal too small"; a question that
   opens then is shown alone and can be answered; growing it back restores the layout.
9. **A panic.** One on the render thread restores the terminal before Rust's report, and the
   shell prompt is usable after it. One on another thread (a background job, the runner)
   leaves the screen as it is and prints nothing over it: the job shows as failed, the run as
   ended with the panic's message (D94).
