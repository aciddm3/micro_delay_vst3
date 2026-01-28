# Micro Delay

## Description

**microdelay** is a delay matrix with two delay lines. There are two delay lines: LA and LB.
There are 15 parameters:

- Send from input to output (Dry Level)
- Send from input to LA
- Balance of send from input to LA
- Send from input to LB
- Balance of send from input to LB
- Send from LA to output
- Balance of send from LA to output
- Send from LA to LA (LA Feedback)
- Send from LA to LB
- LA delay time
- Send from LB to output
- Balance of send from LB to output
- Send from LB to LB (LB Feedback)
- Send from LB to LA
- LB delay time
Every send parameter takes values from -100% to 100%.

Every delay time parameter takes values from 0.025 milliseconds to 16000.00 milliseconds.

Every balance parameter takes values from 0.0 to 1.0.

![Plugin Circuit and future implementation of interface](https://github.com/aciddm3/micro_delay_vst3/tree/master/res/Delay.png)

## TODO

- [ ] Improve GUI:
    - [ ] Realize self-made GUI like Delay.png
    - [ ] add self-made knobs
- [ ] Add Pan knobs (it's useful to make haas effect)
    - [ ] Make honest stereo windows
- [ ] Add Filters

## Building

To compile it to bundle, use this command:
> cargo run --package xtask --release bundle microdelay

## Dependencies

### Crates

- egui,
- nih_plug

### Packages

- gl
- pkg-config
- x11-xcb
