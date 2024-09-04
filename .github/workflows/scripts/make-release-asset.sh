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

bin_name=pdf-tile-viewer
# todo: to mitigate webview failure NO_STRIP is required
env NO_STRIP=1 npm run tauri build --locked -- --target $target

artifact=$bin_name@$TAG

build_dist=src-tauri/target/$target/release

mkdir $build_dist/$artifact

cp ".github/workflows/artifact-assets/notes.txt" "$build_dist/$artifact/"

workdir="$(pwd)"
cd $build_dist

# todo: is not single executable but depends on libpdfium.so now
mkdir -p $artifact/lib/pdfium
os_tag=$3
case $1 in
  ubuntu*)
    cp $bin_name $artifact/
    
    # todo: is not single executable but depends on libpdfium.so now
    curl -LO https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
    mkdir libpdfium
    tar xzf pdfium-linux-x64.tgz -C libpdfium
    cp -r libpdfium/lib $artifact/lib/pdfium/

    asset="$bin_name@$os_tag.tar.gz"
    tar czf ../../$asset $artifact
    ;;
  macos*)
    cp $bin_name $artifact/
    
    # todo: is not single executable but depends on libpdfium.so now
    curl -LO https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-arm64.tgz
    mkdir libpdfium
    tar xzf pdfium-mac-arm64.tgz -C libpdfium
    cp -r libpdfium/lib $artifact/lib/pdfium/

    if [[ -n "${{ $APPLE_CERTIFICATE }}" ]]; then
      echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
      security create-keychain -p "password" build.keychain
      security default-keychain -s build.keychain
      security unlock-keychain -p "password" build.keychain
      security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -A
      codesign --deep --force --verify --sign "$APPLE_CERTIFICATE_NAME" $artifact/$bin_name
      security delete-keychain build.keychain
      rm certificate.p12
    fi

    asset="$bin_name@$os_tag.dmg"
    hdiutil create -volname "$bin_name" \
            -srcfolder $artifact \
            -ov \
            -format UDZO \
            $asset
    ;;
  windows*)
    cp $bin_name.exe $artifact/
    
    # todo: is not single executable but depends on libpdfium.so now
    curl -LO https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz
    7z x pdfium-win-x64.tgz
    7z x pdfium-win-x64.tar -olibpdfium
    # windows requires .dll in bin instead of .dll.lib in lib
    cp -r libpdfium/bin $artifact/lib/pdfium/
    mv $artifact/lib/pdfium/bin $artifact/lib/pdfium/lib

    asset="$bin_name@$os_tag.zip"
    7z a -w ../../$asset $artifact
    ;;
  *)
    echo "OS should be first parameter, was: $1"
    ;;
esac

cd $workdir

if [[ -z "$GITHUB_ENV" ]]
then
  echo "GITHUB_ENV not set, run: gh release upload $TAG src-tauri/target/$asset"
else
  echo "APP_TAG=$TAG" >> $GITHUB_ENV
  echo "APP_ASSET=src-tauri/target/$asset" >> $GITHUB_ENV
fi
