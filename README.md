# build_keyboard_sounds

🌐 **線上即時試聽與自定義混音面板 (Live Showcase & Mixer)**: [https://shadowjohn.github.io/build_keyboard_sounds/](https://shadowjohn.github.io/build_keyboard_sounds/)

原創合成的肥米打字音 WAV 產生器。輸出格式是肥米目前可直接讀取的 RIFF / PCM 16-bit / mono `.wav`。

聲音方向：

- 一般鍵：機械鍵盤青軸感，前段有清楚 click、tactile bump，尾端帶一點塑膠與金屬 ping。
- `backspace.wav`：短促下滑的「咻」聲。
- `enter.wav`：三段式大鍵聲，走「ㄎ-一-尢 / kiang」的感覺。
- `space.wav`：比較低頻的長鍵 thock。

產生預設音效：

```powershell
cargo run --release -- --out .\wavs --preset classic --volume 82
```

批量產生所有音效套件（推薦，全部由演算法合成，分別存在 `wavs/classic/`, `wavs/soft/`, `wavs/crisp/`, `wavs/blue/`, `wavs/retro/`, `wavs/balanced/`）：

```powershell
cargo run --release -- --preset all --volume 82
```

可用 preset：

| preset | 聲音方向 |
| --- | --- |
| `soft` | 7 點低通濾波且降調 10% (0.9x 速度)，極度低沉、安靜的 Thock 靜音線性軸感 |
| `classic` | 微調的高頻瞬態邊緣增強版經典鍵音，乾淨清脆 |
| `crisp` | 大幅升調 15% (1.15x 速度) 並過濾低音，輕快高頻，猶如 Kailh Box 白軸冰裂感 |
| `blue` | 升調 5% (1.05x 速度) 且在起音處融合一組清脆的機械 click jacket 雙重卡榫聲與金屬彈簧餘音，擬真青軸 |
| `retro` | 厚重老打字機鐵錘 clack，搭配下滑退格掃擊音與 Enter 回車鈴聲 |
| `balanced` | 較長尾韻的大鍵回饋與均衡擊鍵音，適合想保留傳統打字感的設定 |
| `all` | 同時輸出上述所有 Preset 到個別子資料夾中 |

單獨產生極擬真青軸的版本：

```powershell
cargo run --release -- --out .\wavs --preset blue --volume 76
```

單獨產生復古打字機版本：

```powershell
cargo run --release -- --out .\wavs --preset retro --volume 82
```

若你有自己錄製且可授權使用的鍵盤聲，可用 `--trace-from` 做私有調音：

```powershell
cargo run --release -- --out .\wavs --preset classic --trace-from .\my_recorded_wavs --volume 82
```

`--trace-from` 只應指向你自己錄製或已取得授權的 WAV。公開版建置與展示頁預設不使用任何第三方音檔。

產生的檔名：

| 檔名 | 肥米用途 |
| --- | --- |
| `1.wav` - `9.wav` | 一般按鍵音，肥米會隨機挑選 |
| `enter.wav` | Enter |
| `delete.wav` | Delete |
| `backspace.wav` | Backspace |
| `space.wav` | Space |

使用方式：把產生出的 `wavs` 目錄放在肥米主程式同一層，然後從肥米右下角選單開啟打字音。

這些聲音是程式合成；此 repo 不包含第三方錄音素材。
