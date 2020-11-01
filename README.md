# rqck_control
control software for Steelseries Cloth Qck written in Rust

## Usage
### Set Intensity
rgame_linux set_intesity INTENSITY

INTENSITY can be any value between 0 and 100. 

### Disable Zones
rgame_linux disable ZONE 

ZONE can be "lower" or "upper". You can disable both zones by providing both at the same time. Ordering is not important.
rgame_linux disable lower => disables the lower zone
rgame_linux disable upper => disables the upper zone
rgame_linux disable lower upper => disables both zones.

### Switching Modes
To "Steady" currently not supported :(
To "ColorShift" currently not supported :(
To "Multi Color Breathe" currently not supported :(

### Steady Mode
* Setting color: Currently not supported :(

### Color Shift
* Adding keyframe + setting color: Currently not supported :(
* Changing speed: Currently not supported :(

### Multi Color Breathe
* Adding keyframe + setting color: Currently not supported :(
* Changing speed: Currently not supported :(
