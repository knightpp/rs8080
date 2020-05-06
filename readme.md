Intel 8080 emulator and space invaders arcade machine.
`rs8080-space-invaders` uses SDL2 for rendering. Binaries can be built on Linux and Windows.
## Build
`bundlerom` feature includes rom files in a binary file.
### With sound
You can optionally enable 'sound' feature, but you will need [SDL_mixer](https://www.libsdl.org/projects/SDL_mixer/) development libraries (only `SDL_mixer.dll` and `SDL_mixer.lib`) to be placed in `rs8080-space-invaders/SDL2_mixer/64/` or 86 folder. Also, you will need some .wav files. Place sounds in `rs8080-space-invaders/sounds/`.

```cargo r --bin rs8080-space-invaders --features "sound bundlerom"```

### No sound

```cargo r --bin rs8080-space-invaders --features "bundlerom"```
## Intel 8080 emulation TODOs
Some things may never be implemented
- [ ] Implement DAA and aux carry 
- [ ] A lot of tests
- [X] Is `cpudiag` succeeds?

## Space Invaders TODOs
- [X] Is space invaders playable? 
- [X] Config file, keyboard bindings for two players
- [X] Sounds

## Used resources
- http://computerarcheology.com/Arcade/SpaceInvaders
- http://demin.ws/projects/radio86/info/kr580/i8080.html
- http://www.emulator101.com/
- https://bluishcoder.co.nz/js8080/
- https://www.walkofmind.com/programming/side/hardware.htm
- Book: intel 8080 Assembly Language Programming Manual (Rev. B)
