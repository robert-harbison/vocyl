# Vocyl

Vocyl is a work-in-progress tool for text-to-speech and speech-to-text on your computer.

# Support

- Windows
- Linux
    - X11
    - Wayland best effort
- Mac

# TODO: Temp TODO

STORIES

- We may need to add a option to unload kokoro or sherpa between runs if the user doesn't use it often.
- Add a open at startup option
- Allow user to adjust position of the indicator overlay. We could make a button in settings which shows it and the user can drag it to the desired position.
    - Also allow them to hide it entirely.
- Make settings not need a save button. Just save on change?
- Make schema version 1 since we don't have a actual release or users yet.
- Make a popup warning (and option to disable) when the user tries to run TTS with no text selected.

IDEAS:

- Can we add a right click context option to like all selected text consistently to read with TTS? This would be optionally enabled for the user.
- Can we add readback speed option?
- Make tauri window overlay click through

TASKS

- Add rust linting

BUGS

- Fix crash when / freeze when hitting TTS button with no text selected.
