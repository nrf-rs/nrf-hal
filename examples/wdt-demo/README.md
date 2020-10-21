# Watchdog timer demo - Timer

This demonstrates the how the entire device can be set to automatically reset if certain conditions are not met within a certain period of time. In this case you have to press all 4 buttons at least once within a 5 second period to prevent a reset. 

> Can you spot an opportunity to crash the program? 
> 
> If you mash the buttons as the time approaches zero you will encounter an 'attempt to subtract with overflow' panic at `main.rs:205` which is ultimately cleared by the watchdog timer. This demonstrates the ability to recover from a panic. Use `probe-run` to see the actual panic message.

## How to run 

If using `cargo-embed`, just run

```console
$ cargo embed --release --target=thumbv7em-none-eabihf
```