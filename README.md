# MHF Launcher

A custom Monster Hunter Frontier launcher.

## Features

- Boots _much_ faster than the original
- Includes a 'Modern' and a 'Classic' style
    <p align="center">
        <img src="./docs/main.png" width="49%">
        <img src="./docs/main-classic.png" width="49%">
    </p>
- Server management without editing `/etc/hosts`, including an option to choose multiple installations for different servers
    <p align="center">
        <img src="./docs/server-list.png" width="49%">
        <img src="./docs/server-edit.png" width="49%">
    </p>
- Multi-language support (currently EN and JP)
- Allows you to download your character data
- Allows servers to customize what icons and messages their users will see
- Reauthentication when token expires (while still in the client)

## Planned Features

- More styles! (PS4 Launcher, non-ZZ themes)
- Game updater/patcher
- More server-related 'commands' that are not supported in-game: Renaming character, save uploading, etc
  - These will _all_ be opt-in by the server host

## Usage

Download the [latest release](https://github.com/rockisch/mhf-launcher/releases/latest) and drop it in the same folder as `mhf.exe`.

Notice that the launcher requires your `mhfo-hd.dll` to be decrypted. Most community releases in the wild already do that, so it probably won't be an issue.

## Supported Servers

For now, only servers using the latest version of [this fork](https://github.com/rockisch/Erupe-1) will work with the new launcher.

## Contributing

- If you feel there's a inconsistency with the original launcher or you want to improve upon the existing designs, feel free to open a PR or an issue.
- Language translation help is always appreaciated!
  - If you want help with translation to an already supported language, just edit corresponding the `locale/<id>.ftl` file.
  - If you want to translate the laucher to a new language, duplicate `locale/en.ftl` and rename it to the correct language code. It's Ok to not fully translate the file, the launcher already falls back to english when it can't find a translation.
