# CHIP-8 Technical Specifications

## Display
- 64x32 monochrome display
- Sprites are 8 pixels wide and 1 to 16 pixels tall

## Registers
- **16 8-bit general purpose registers** (V0 - VF)
  - VF doubles as a flag register for overflow operations
- **16-bit program counter** (PC)
- **16-bit register** used as a pointer for memory access (I register)

## Memory
- **4KB RAM** (4096 bytes)

## Stack
- **16-bit stack** for calling and returning from subroutines

## Input
- **16-key keyboard** input

## Timers
Two special registers that decrease each frame and trigger upon reaching 0:
- **Delay timer**: Used for time-based game events
- **Sound timer**: Used to trigger the audio beep
