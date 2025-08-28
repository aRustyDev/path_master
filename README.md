# path_master
a rewrite of the /usr/libexec/path_helper utility. Meant to be installed and explicitly run from .zprofile or .zshrc

GOALS:
1. manage more paths.d/ and ENV vars than just PATH & MANPATH
2. create the paths to all the directories in the paths.d/ files
3. read json blob from stdin and create config files (blank)

## Usage

```sh
path_master completion <shell>

# Outputs *PATH vars (as map) from /etc/*path.d/*
path_master

# Create the DIRS in the paths
path_master -p

# Create the FILES in the paths
echo '[{key:value}]' | path_master -p
```
