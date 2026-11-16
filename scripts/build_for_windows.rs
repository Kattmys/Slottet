#!/bin/bash
cargo build -p client --target x86_64-pc-windows-gnu --release

mkdir slottet_windows

mv ./target/x86_64-pc-windows-gnu/release/*.exe slottet_windows
cp -r assets slottet_windows

7z a slottet_windows.7z slottet_windows
rm -rf slottet_windows
