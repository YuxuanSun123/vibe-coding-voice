$env:CARGO_HOME = 'c:\Users\ROG\Downloads\AI VOICE\vibe-coding-voice-app\.cargo-home'
$env:RUSTUP_HOME = 'c:\Users\ROG\Downloads\AI VOICE\vibe-coding-voice-app\.rustup-home'
$env:Path = "c:\Users\ROG\Downloads\AI VOICE\vibe-coding-voice-app\.cargo-home\bin;$env:Path"
$env:CARGO_HTTP_CHECK_REVOKE = 'false'

Set-Location $PSScriptRoot
cargo run --bin vibe-coding-voice-native
