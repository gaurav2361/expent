{
  pkgs,
  android-nixpkgs,
  system,
}:

let
  arch = if pkgs.stdenv.isAarch64 then "arm64-v8a" else "x86_64";

  # Build the Android SDK
  androidSdkBuilder = android-nixpkgs.sdk.${system};
  androidSdk = androidSdkBuilder (
    sdkPkgs: with sdkPkgs; [
      cmdline-tools-latest
      platform-tools
      emulator

      build-tools-36-0-0
      platforms-android-36

      build-tools-35-0-0
      platforms-android-35

      build-tools-34-0-0
      platforms-android-34

      ndk-27-1-12297006
      cmake-3-22-1

      (
        if pkgs.stdenv.isAarch64 then
          system-images-android-34-google-apis-arm64-v8a
        else
          system-images-android-34-google-apis-x86-64
      )
    ]
  );

  javaVersion = pkgs.jdk17;

  runEmulatorScript = pkgs.writeShellScriptBin "run-emulator" ''
    export ANDROID_AVD_HOME="$HOME/.android/avd"
    mkdir -p "$ANDROID_AVD_HOME"

    if ! avdmanager list avd | grep -q "macos_emulator"; then
      echo "Creating Android 34 Emulator for ''${arch}..."
      echo "no" | avdmanager create avd \
        --force \
        --name macos_emulator \
        --package "system-images;android-34;google_apis;''${arch}" \
        --device "pixel"
    fi

    echo "Starting Emulator on macOS..."
    EMULATOR_BIN="${androidSdk}/share/android-sdk/emulator/emulator"
    $EMULATOR_BIN -avd macos_emulator -dns-server 8.8.8.8 -gpu host &
  '';

in
{
  packages = [
    pkgs.watchman
    javaVersion
    androidSdk
    pkgs.cocoapods
    runEmulatorScript
  ];

  env = {
    ANDROID_HOME = "${androidSdk}/share/android-sdk";
    ANDROID_SDK_ROOT = "${androidSdk}/share/android-sdk";
    ANDROID_NDK_HOME = "${androidSdk}/share/android-sdk/ndk/27.1.12297006";
    JAVA_HOME = javaVersion.home;
  };

  shellHook = ''
    export ANDROID_AVD_HOME="$HOME/.android/avd"

    echo "-------------------------------------"
    echo "🍎 macOS React Native Environment Ready"
    echo "   Loaded: SDK 36/35, NDK 27 & CMake 3.22.1"
    echo "-------------------------------------"
  '';
}
