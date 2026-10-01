# 📸 macOS Screen Shot (Rust)

macOS için geliştirilmiş, **iShot** ve **Shottr** kalitesinde, ultra hızlı, hafif ve zengin özelliklere sahip ekran alıntısı ve işaretleme (screenshot & annotation) aracı.

Tamamen **Rust** ile geliştirilmiş olup işletim sisteminin yerel grafik ve ekran yakalama API'lerini kullanır. Sıfır çöp toplayıcı (GC) gecikmesi ile 60/120 FPS akıcılıkta çalışır.

---

## ✨ Özellikler

### 1. 🔍 Piksel Büyüteci (Loupe & Color Picker)
- İmlecin etrafındaki pikselleri **8x büyütülmüş ızgara** üzerinde gösterir.
- Merkezdeki hedef pikseli vurgular.
- Anlık piksel rengini **HEX (`#FF3B30`)** ve **RGB (`255, 59, 48`)** formatında görüntüler.
- **`C` tuşuyla** anında rengin HEX kodunu panoya kopyalar.

### 2. 🪟 Akıllı Pencere Yakalama (Smart Window Snapping)
- Fare açık bir pencerenin üzerine geldiğinde pencereyi otomatik algılar ve vurgular.
- **`Boşluk (Space)` tuşuna** basarak tek tıkla pencerenin tam sınırlarını seçebilirsiniz.

### 3. ✏️ Zengin İşaretleme Araçları (Annotation Toolkit)
- 🔲 **Dikdörtgen (`R`)**: Vurgulamak istediğiniz alanlar için kenarlıklı veya hafif dolgulu kutu.
- ⭕ **Daire / Elips (`O`)**: Oval veya dairesel alan vurgulama.
- ➔ **Vektörel Ok (`A`)**: Yönlendirme ve dikkat çekme okları.
- ✎ **Kalem (`P`)**: Serbest çizim ve karalama.
- ① **Adım Numaratörü (`N`)**: Tıkladıkça otomatik artan `1`, `2`, `3` rozetleri (eğitim ve dokümantasyonlar için ideal).
- ▦ **Buzlama & Mozaik (`B`)**: Şifreler, e-postalar veya hassas veriler için gerçek pikselli mozaik filtresi.
- ▨ **Vurgulayıcı (`H`)**: Yarı saydam marker çizimi.
- 🎨 **Apple Renk Paleti**: Kırmızı, Mavi, Yeşil, Sarı, Mor, Beyaz.
- ↩ **Geri Al (`Cmd+Z`)** & 🗑️ **Temizle**.

### 4. 📌 Ekrana Sabitle (Pin to Screen - Floating Window)
- iShot'ın en popüler özelliği! Seçtiğiniz ekran alıntısını masaüstünde her şeyin üzerinde yüzen (`always-on-top`) şeffaf bir pencereye dönüştürür.
- İstediğiniz yere sürükleyebilir, fare tekerleğiyle yakınlaştırıp uzaklaştırabilir, `Cmd+C` ile kopyalayabilir veya çift tıklayarak kapatabilirsiniz.

### 5. 📋 Dışa Aktarma Seçenekleri
- **Panoya Kopyala (`Enter` / `Cmd+C`)**: Çizimlerle birlikte görüntüyü doğrudan panoya atar.
- **Masaüstüne Hızlı Kaydet (`Space`)**: `~/Desktop/Ekran Resmi YYYY-MM-DD.png` olarak anında kaydeder.
- **Farklı Kaydet (`Cmd+S`)**: Dosya konumu ve biçimi (PNG / JPEG) seçerek kaydeder.

---

## ⌨️ Klavye Kısayolları

| Kısayol | İşlev |
| :--- | :--- |
| **`Enter` / `⌘ + C`** | Seçimi çizimlerle panoya kopyalar ve çıkar |
| **`⌘ + S`** | Farklı kaydet iletişim kutusu açar |
| **`Space`** | Hızlı masaüstüne kaydet (veya pencere üzerine gelindiğinde pencereyi seçer) |
| **`C`** | Büyüteç altındaki pikselin HEX renk kodunu kopyalar |
| **`V`** | Seçim / Taşıma aracı |
| **`R`** | Dikdörtgen aracı |
| **`O`** | Daire / Elips aracı |
| **`A`** | Ok aracı |
| **`P`** | Kalem aracı |
| **`N`** | Adım numaratörü (1, 2, 3...) |
| **`B`** | Buzlama / Mozaik aracı |
| **`H`** | Vurgulayıcı (Highlighter) |
| **`⌘ + Z`** | Son çizimi geri al |
| **`Esc`** | Alıntıyı iptal et ve kapat |

---

## 🚀 Kurulum ve Çalıştırma

### Gereksinimler
- macOS (Apple Silicon M1/M2/M3/M4 veya Intel)
- Rust & Cargo (1.80+)

### Çalıştırma
```bash
# Projeyi derleyin ve ekran alıntısı modunu başlatın
cargo run --release
```

### Sabitleme (Pin) Modunu Doğrudan Çalıştırma
Dilerseniz mevcut bir görseli doğrudan ekrana sabitleyebilirsiniz:
```bash
cargo run --release -- --pin /path/to/image.png
```

---

## 🔐 macOS İzinleri
macOS'ta uygulamanın ekranı okuyabilmesi için **Ekran Kaydı (Screen Recording)** iznine ihtiyacı vardır:
1. **Sistem Ayarları (System Settings)** > **Gizlilik ve Güvenlik (Privacy & Security)** bölümüne gidin.
2. **Ekran Kaydı (Screen Recording)** sekmesinde terminalinize veya oluşturduğunuz uygulamaya izin verin.

---

## 🛠️ Mimari ve Kullanılan Teknolojiler

- **[xcap](https://crates.io/crates/xcap)**: Yüksek performanslı, yerel CoreGraphics ve ScreenCaptureKit ekran ve pencere yakalama.
- **[egui](https://crates.io/crates/egui) & [eframe](https://crates.io/crates/eframe)**: Donanım hızlandırmalı (Metal GPU), anlık tepkili tam ekran şeffaf arayüz.
- **[arboard](https://crates.io/crates/arboard)**: Platformlar arası yerel pano (clipboard) yönetimi.
- **[imageproc](https://crates.io/crates/imageproc) & [image](https://crates.io/crates/image)**: Anti-aliased çizimler, mozaik algoritması ve Retina çözünürlükte kırpma.
- **[rfd](https://crates.io/crates/rfd)**: macOS yerel dosya kaydetme pencereleri.
