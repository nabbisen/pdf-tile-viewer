#!/usr/bin/env bash
# Builds the release and creates an archive and optionally deploys to GitHub.
set -ex

if [[ -z "$GITHUB_REF" ]]
then
  echo "GITHUB_REF must be set"
  exit 1
fi
# Strip (repo)-refs/tags/ from the start of the ref.
TAG=${GITHUB_REF#*/tags/}

host=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")
target=$2
if [ "$host" != "$target" ]
then
  export "CARGO_TARGET_$(echo $target | tr a-z- A-Z_)_LINKER"=rust-lld
fi
export CARGO_PROFILE_RELEASE_LTO=true

BIN_NAME=pdf-tile-viewer
# todo: to mitigate webview failure NO_STRIP is required
env NO_STRIP=1 npm run tauri build --locked -- --target $target

cd src-tauri/target/$target/release

mkdir $BIN_NAME-main
os_tag=$3
case $1 in
  ubuntu*)
    # asset="$BIN_NAME-$os_tag-$TAG"
    asset="$BIN_NAME-$os_tag"
    mv $BIN_NAME $asset
    ;;
  macos*)
    asset="$BIN_NAME-$os_tag"
    mv $BIN_NAME $asset
    ;;
  windows*)
    asset="$BIN_NAME-$os_tag.exe"
    mv $BIN_NAME.exe $asset
    ;;
  *)
    echo "OS should be first parameter, was: $1"
    ;;
esac

cd ../..

if [[ -z "$GITHUB_ENV" ]]
then
  echo "GITHUB_ENV not set, run: gh release upload $TAG src-tauri/target/$asset"
else
  echo "APP_TAG=$TAG" >> $GITHUB_ENV
  echo "APP_ASSET=src-tauri/target/$target/release/$asset" >> $GITHUB_ENV
fi
