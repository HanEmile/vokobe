# Vokobe

A minimal static site generator tailored to my needs.

CI: [https://hydra.emile.space/project/vokobe](https://hydra.emile.space/project/vokobe)

## Installation

Install my-project with npm

```bash
; cargo build --release
```
    
## Usage/Examples

```javascript
; ./target/release/vokobe --help
vokobe 0.1.0
A static site generator

USAGE:
    vokobe [FLAGS] <in-path> <out-path> <site-name>

FLAGS:
    -a, --analytics    Activate sending analytics to stats.emile.space
    -h, --help         Prints help information
    -V, --version      Prints version information

ARGS:
    <in-path>      Input path
    <out-path>     Output path
    <site-name>    Site name (e.g. emile.space)
```


## Deployment

The following subsections contain some example for small shell scripts that might be useful for Deployment.

### build.sh

Remove the output dir, build it from scratch and update the perms.

I'm actually considering rebuilding vokobe with incremental builds in mind, as it can take a bit to create some really large projects.

```bash
rm -rf out/
vokobe -a ./in ./out emile.space
chmod -R +r out/
```

### sync.sh

Syncronize the generated output to the remote host for hosting it.

```bash
rsync -avz --delete <out-path>/* <user>@<host>:<path>
```

### publish.sh

Build and Syncronize.

```bash
./build.sh
./sync.sh
```

### host.sh

Host the local version

```bash
python3 -m http.server 8081 -d <outpath>/ -b 0.0.0.0
```

### watchbuild.sh

rebuild on changes

```bash
#! /usr/bin/env nix-shell
#! nix-shell -i bash -p fd entr

while sleep 0.5; do
  fd . in | entr -d ./build.sh
done
```

### local.sh

run a script updating it on changes and one hosting the output.

```bash
sh ./watchbuild.sh &
sh ./host.sh
```


## Contributing

Send patches!
