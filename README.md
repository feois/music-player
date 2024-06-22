## Description

This project is about an open-source music player built with Rust (backend) and Godot (frontend).

## Getting Started

### Installing

You can simply download from [here](https://github.com/feois/music-player/releases)

## How to compile

### Dependencies

1. You need Rust and Godot to compile this project.

2. Before compiling, you need to use Godot to open the project (src/gui/project.godot) to setup the export presets.

3. You need to go to Project/Export and add a new preset

![image](https://github.com/feois/music-player/assets/68548170/cf1e32e2-0903-44b7-a227-49f6cba4c69e)
![image](https://github.com/feois/music-player/assets/68548170/4a1b8c65-7d7d-43fc-b7d9-3ab55bb4cdd0)

4. You must rename the preset to the os name in [Rust target triples](https://doc.rust-lang.org/stable/rustc/platform-support.html#tier-1-with-host-tools) (e.g. "windows" or "linux")
5. (Optional) You can check on Embed PCK to export in only one binary
6. Simply run `cargo build -r` (You have to add `--no-default-features` if you are not using X11)

### Compiling for Linux

After following all the steps above, you're done!

### Compiling for Windows

Oh well, that's unfortunate to hear. You need a custom Godot export template due to [a Godot bug](https://github.com/godotengine/godot-proposals/issues/9932).

This bug is fixed in Godot v4.3 (which is not released yet at the moment of writing) so while you can technically use Godot v4.3 beta, I do not recommend it as there is [a change](https://github.com/godotengine/godot-proposals/issues/9945) that can break the UI.

Therefore, the recommended way is to build a custom export template yourself with the [bugfix](https://github.com/godotengine/godot/pull/91147) cherry-picked or [download my pre-built template](https://github.com/feois/music-player/releases)

### How exactly do I compile a custom export template???

1. `git clone https://github.com/godotengine/godot.git`
2. `git switch 4.2`
3. `git remote add bugfix-fork https://github.com/bruvzg/godot.git`
4. `git fetch bugfix-fork`
5. `git switch con_redir_3`
6. `git checkout 4.2`
7. `git cherry-pick con_redir_3`
8. `scons target=template_release disable_3d=yes module_camera_enabled=no module_navigation_enabled=no module_raycast_enabled=no module_multiplayer_enabled=no module_openxr_enabled=no platform=windows`
9. Your template should be in bin/
10. Enter the path to your template in the Godot export window
![image](https://github.com/feois/music-player/assets/68548170/c6401b85-f128-41b5-8c84-d22a6a491aff)


## Help

Known bugs:
- The GUI might not be successfully closed sometimes, you will need to use Task Manager (or anything similar) to terminate the program

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
