## Description

This project is about an open-source music player built with Rust (backend) and Godot (frontend).

## Getting Started

### Installing

You can simply download from [here](https://github.com/feois/music-player/releases)

### How to use

This application is separated into two different program, the backend (Rust) and the frontend (Godot).

The backend can run even when the frontend is closed, but you need the frontend to manage playlists.

The backend can listen to keyboard from anywhere so even when the backend is closed you can control some functionalities.

The controls listed below that is tagged GUI (e.g. **press some key (GUI)**) will only works when the frontend window is focused while the controls tagged Global (e.g. **press some key (Global)**) will work regardless if frontend window is focused.

The global controls only listen to keys on the left side (i.e. Shift left, Control left, Alt left).

### The frontend (GUI)

![image](https://github.com/feois/music-player/assets/68548170/db915ab0-58ca-4fef-8e26-b619374bf1eb)

- You can **press W or Up (GUI)** or **press S or Down (GUI)** to move up or down when the library is focused
  - Similar feature exists for the playlist
- You can **double click (GUI)** or **press Enter (GUI)** while a song is selected to **add it into playlist** and **play it instantly**
  - The **play** symbol beside the song works similarly
- Or you can also **press shift (GUI)** while you **double click (GUI)** or **press Enter (GUI)** to just **add into playlist** and not play it
  - The **plus** symbol beside the song works similarly
- You can **press Space (GUI)** to **search the library**
- You can **right click the library (GUI)** to change to group by artists or albums

![image](https://github.com/feois/music-player/assets/68548170/07bf0d17-0ddd-4a5b-8464-d57a66931171)

- When you play a song, there will be a new section under playlist that shows title, artist, album and lyrics
- You can **press H (GUI)** to hide this section
- You can also **press P (GUI)** to hide playlists
- You can **drag the circled area (GUI)** to change the size of each section
- Volume controls
  - You can **click the volume button (GUI)** or **press Alt+M (Global)** to **mute or unmute**
  - You can **click (GUI)** the volume bar to **set volume**
    - You can also **click and drag (GUI)** the volume bar to **change volume**
  - You can **scroll up or down (GUI)** or **press Alt+Up or Alt+Down (Global)** to **change volume by a step** (like +5% or -5%)
- Playback controls
  - You can **click the progress bar (GUI)** to **jump to that specific position of the song**
    - You can also **click and drag the progress bar (GUI)**
  - You can **click the play symbol (GUI)** or **press Alt+Space (Global)** to **play or pause or resume song**
  - You can **click the rewind symbol left to the play symbol (GUI)** or **press Alt+Left (Global)** to **rewind by a set duration**
  - You can **click the fast forward symbol right to the play symbol (GUI)** or **press Alt+Right (Global)** to **fast forward by a set duration**
  - You can **click the jump to beginning symbol left to the rewind symbol (GUI)** or **press Alt+Ctrl+Left (Global)** to **jump to the beginning of the song** or to de facto replay the song
  - You can **click the jump to end symbol right to the fast forward symbol (GUI)** or **press Alt+Ctrl+Right (Global)** to **jump to the end of the song** or to de facto skip to next song
  - You can **click the history symbol left to the jump to beginning symbol (GUI)** or **press Alt+Ctrl+Up (Global)** to **go back to previous song in history**, this feature works similarly to the history feature in browsers
  - You can **click the stop symbol right to the jump to end symbol (GUI)** or **press Alt+Shift+Space (Global)** to **stop the playback**
- Playlist controls
  - You can **click the plus symbol (GUI)** to **add a new playlist**
  - You can **click the T symbol (GUI)** to **rename playlist**
  - You can **click the folder symbol (GUI)** to **import a playlist**, note that you might need to reload library after importing
  - You can **click the disk symbol (GUI)** to **export a playlist**
  - You can **click the X symbol (GUI)** to **delete a playlist**, you cannot delete when only one playlist presents
  - You can select a song and **press Delete (GUI)** to **remove a song from playlist**
  - You can select a song, then **click and drag (GUI)** to **change the song's position** in the playlist
  - You can select a song and **double click (GUI)** or **press Enter (GUI)** to **play it instantly**
- Automatic playback controls
  - You can **click the Repeat symbol (GUI)** or **press Alt+R (Global)** to **toggle the repeat mode**, the current symbol represents the repeat mode currently used
    - Repeat All mode: The playlist is repeated when finished
    - Repeat One mode: The same song is replayed again and again
    - No Repeat mode: The playback will stop when the playlist is finished
    - Stop Every Song mode: The playback will stop every time the song is finished
  - You can **click the Shuffle symbol (GUI)** or **press Alt+Shift+R (Global)** to **turn on or off the shuffle mode**, the dimmed shuffle symbol means no shuffling is currently performed
    - Shuffling is fair, every song in the playlist will be guaranteed to be played once before the playlist is considered finished
    - If no shuffling is enabled, the next song will be the song below the currently played song in the order of the playlist
- You can **click the reset symbol in the top left section (GUI)** to **reload the library**

![image](https://github.com/feois/music-player/assets/68548170/bf20171e-e0fd-4543-a704-eda8775b553d)

- You must **click Save** before closing the settings for the changes to take effect
- You can only **click Discard** to close the settings (and discard any changes at the same time)
- You can **change the libary path** in **settings**, you can either **click the path and type the desired path** or **press the folder symbol** to bring up a window that helps selecting directory
  - Note that you will need to **reload library** to take effect

### The backend

- Closing the frontend window will not close the backend process
- You can **press Alt+E (Global)** or **click the X symbol on top left (GUI)** to **close the backend process** (and also the frontend window)
- You can **press Alt+C (Global)** to **close or open the frontend window**

### Synchronized lyrics / Floating lyrics

Floating lyrics is a feature that is **only available for x11**. The feature currently **only supports the SYLT frame in id3 tags (tags used by MP3)**.

Floating lyrics are enabled by default and activates automatically when you play a song with SYLT frame. Floating lyrics will not disappear when the frontend window is closed and will stay on top of all windows.

![image](https://github.com/feois/music-player/assets/68548170/b090a0e2-5ddf-4b3c-9b26-0e08fae69207)
![image](https://github.com/feois/music-player/assets/68548170/bf02a58f-a5a8-4f1b-9a82-cb628dc6c98b)
![image](https://github.com/feois/music-player/assets/68548170/619f8c02-f8ba-4735-b3c8-2e89658bfa43)

You can **press Alt+H (Global)** to **show or hide the floating lyrics**.

You can **press Alt+L+1 or Alt+L+2 or ... or Alt+L+9 (Global)** to **change the position of floating lyrics** (Note that you need to play the song after you change position to update the floating lyrics)

![image](https://github.com/feois/music-player/assets/68548170/3c767063-27c4-403e-ae8e-d654696ae260)
![image](https://github.com/feois/music-player/assets/68548170/d6fa0b5e-5971-44cb-9920-45239e59fe40)
![image](https://github.com/feois/music-player/assets/68548170/0eb58f02-996d-454a-a2a2-0456ea2ca3e8)
![image](https://github.com/feois/music-player/assets/68548170/f6f3796e-e003-4790-8f21-bb6593f9d072)
![image](https://github.com/feois/music-player/assets/68548170/30344092-1c82-4aae-9dee-79e71a5ad19c)
![image](https://github.com/feois/music-player/assets/68548170/a8fedac4-08a1-4c0c-a413-bce4dd4be26d)
![image](https://github.com/feois/music-player/assets/68548170/dfe7af27-ec82-427d-b392-3d62b7213d72)
![image](https://github.com/feois/music-player/assets/68548170/3029f956-a5ff-4b15-ac4c-b6bc73825b64)
![image](https://github.com/feois/music-player/assets/68548170/f2926f30-a6dc-4d39-9722-4e6e3f22088e)

You can change the margin of the floating lyrics from the edge of screen in settings of the frontend window.


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


## FAQ

### I don't like the color scheme

It is a planned feature to allow custom themes, but you can manually change the theme (It is an easy task!) in Godot and compile yourself at the moment

### I don't like the floating lyrics' font

You can change it in `src/lyrics.rs` and (again) compile yourself, note that it has to be in the format of [XLFD](https://en.wikipedia.org/wiki/X_logical_font_description)

### I don't like the keybinds

It's planned to allow changing keybinds but as I am happy with the current configuration, it will not be a priority for me, you are welcome to make a pull request tho :)

You can change it in `src/main.rs` and compile yourself

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
