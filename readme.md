# Mintorrent
### this is a small bittorent reader that works from the terminal easy easy

## Description!!
so how this works is it decodes the torrent file, makes a url from it and connects to the peers 
in that torrent. then it sends requests for those peers to see the pieces they have.

Then it just sends a TCP stream to those peers and then downloads the pieces from them, writing the pieces to file. easy

### I made this project for horizons. and why a bittorent reader becasuse i felt like it

### Features:
1- easily accessible from anywhere in the pc using the terminal.\
2- single file downloads. there is multiple files but its really buggy i gave up\
3- you can run a debug style but you have to have rust installed. otherwise no need


### Installation:
Easy just run this command
```powershell
iwr -useb https://github.com/MintyEcho/El-bittorento/releases/latest/download/mintorrent.exe -OutFile "$env:LOCALAPPDATA\Microsoft\WindowsApps\mintorrent.exe"; Unblock-File "$env:LOCALAPPDATA\Microsoft\WindowsApps\mintorrent.exe"
```

that's it. 

### There are no dependencies. just windows

### Usage: 

open a git bash or cmd or powershell where the torrent file exists. and write it in this formula

mintorrent (torrent name).torrent (path of insallation)


