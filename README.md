# 📸 macOS Screen Shot (Rust)

macOS için geliştirilmiş, **iShot** ve **Shottr** kalitesinde, ultra hızlı, hafif ve zengin özelliklere sahip ekran alıntısı, menubar ve işaretleme (screenshot & annotation) aracı.

Tamamen **Rust** ile geliştirilmiş olup işletim sisteminin yerel grafik ve ekran yakalama API'lerini kullanır. macOS Menubar'da (üst durum çubuğunda) yer alır, arka planda sıfır CPU ve minimum RAM (~20 MB) ile uyur ve global kısayollara anında tepki verir.

---

## ✨ Özellikler

### 1. 🎛️ macOS Menubar (Durum Çubuğu Simgesi)
- Ekranın en üstündeki Menubar'da kamera simgesi `[📷]` ile yer alır.
- Menüden tek tıkla:
  - 📸 **Ekran Alıntısı Al**
  - 🖥️ **Tüm Ekranı Yakala**
  - 📌 **Görsel Sabitle...**
  - ⚙️ **Ayarlar & Kısayollar...**
  - ❌ **Çıkış**

### 2. ⌨️ Özelleştirilebilir Global Kısayollar (Settings UI)
- Menubar'daki **"Ayarlar & Kısayollar..."** menüsünden tüm global kısayolları dilediğiniz tuş kombinasyonuyla değiştirebilirsiniz:
  - **Bölge Ekran Alıntısı**: Varsayılan `⌥A` (Option+A) — Dilerseniz `⌘⇧A`, `⌃⌥A` veya istediğiniz tuşa ayarlayın.
  - **Tam Ekran Yakalama**: Varsayılan `⌥S` (Option+S).
  - **Ekrana Sabitle**: Varsayılan `⌥P` (Option+P).
- Ayarlar otomatik olarak `~/.config/macos_screenshot/config.json` dosyasına kaydedilir ve anında yürürlüğe girer.
- Otomatik panoya kopyalama, otomatik klasöre kaydetme ve görsel formatı (PNG / JPEG) seçimleri.

### 3. 🔍 Piksel Büyüteci (Loupe & Color Picker)
- İmlecin etrafındaki pikselleri **8x büyütülmüş ızgara** üzerinde gösterir.
- Merkezdeki hedef pikseli vurgular.
- Anlık piksel rengini **HEX (`#FF3B30`)** ve **RGB (`255, 59, 48`)** formatında görüntüler.
- **`C` tuşuyla** anında rengin HEX kodunu panoya kopyalar.

### 4. 🪟 Akıllı Pencere Yakalama (Smart Window Snapping)
- Fare açık bir pencerenin üzerine geldiğinde pencereyi otomatik algılar ve vurgular.
- **`Boşluk (Space)` tuşuna** basarak tek tıkla pencerenin tam sınırlarını seçebilirsiniz.

### 5. ✏️ Zengin İşaretleme Araçları (Annotation Toolkit)
- 🔲 **Dikdörtgen (`R`)**: Vurgulamak istediğiniz alanlar için kenarlıklı veya hafif dolgulu kutu.
- ⭕ **Daire / Elips (`O`)**: Oval veya dairesel alan vurgulama.
- ➔ **Vektörel Ok (`A`)**: Yönlendirme ve dikkat çekme okları.
- ✎ **Kalem (`P`)**: Serbest çizim ve karalama.
- ① **Adım Numaratörü (`N`)**: Tıkladıkça otomatik artan `1`, `2`, `3` rozetleri (eğitim ve dokümantasyonlar için ideal).
- ▦ **Buzlama & Mozaik (`B`)**: Şifreler, e-postalar veya hassas veriler için gerçek pikselli mozaik filtresi.
- ▨ **Vurgulayıcı (`H`)**: Yarı saydam marker çizimi.
- 🎨 **Apple Renk Paleti**: Kırmızı, Mavi, Yeşil, Sarı, Mor, Beyaz.
- ↩ **Geri Al (`Cmd+Z`)** & 🗑️ **Temizle**.

### 6. 📌 Ekrana Sabitle (Pin to Screen - Floating Window)
- iShot'ın en popüler özelliği! Seçtiğiniz ekran alıntısını masaüstünde her şeyin üzerinde yüzen (`always-on-top`) şeffaf bir pencereye dönüştürür.
- İstediğiniz yere sürükleyebilir, fare tekerleğiyle yakınlaştırıp uzaklaştırabilir, `Cmd+C` ile kopyalayabilir veya çift tıklayarak kapatabilirsiniz.

### 7. 📋 Dışa Aktarma Seçenekleri
- **Panoya Kopyala (`Enter` / `Cmd+C`)**: Çizimlerle birlikte görüntüyü doğrudan panoya atar.
- **Masaüstüne Hızlı Kaydet (`Space`)**: `~/Desktop/Ekran Resmi YYYY-MM-DD.png` olarak anında kaydeder.
- **Farklı Kaydet (`Cmd+S`)**: Dosya konumu ve biçimi (PNG / JPEG) seçerek kaydeder.

---

## ⌨️ Klavye Kısayolları

### Global Kısayollar (Arka Planda Dinlenir & Ayarlanabilir)
| İşlev | Varsayılan Kısayol | Açıklama |
| :--- | :--- | :--- |
| **Ekran Alıntısı** | **`⌥ + A`** | Tam ekran overlay açar, seçim ve çizim yaptırır |
| **Tam Ekran** | **`⌥ + S`** | Tüm ekranı tek tıkla yakalar ve panoya/klasöre kaydeder |
| **Görsel Sabitle** | **`⌥ + P`** | Dosya seçtirerek masaüstüne sabitler |

### Alıntı Esnasındaki Kısayollar
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

## 🚀 Çalıştırma Seçenekleri

### 1. Menubar Servisi Olarak Başlatma (Tavsiye Edilen)
En üst menü çubuğuna `[📷]` simgesini yerleştirir ve global kısayolları dinlemeye başlar:
```bash
./target/release/macos_screenshot
# veya
cargo run --release
```

### 2. Doğrudan Ayarlar Penceresini Açma
Kısayolları ve klasör tercihlerini yapılandırmak için:
```bash
cargo run --release -- --settings
```

### 3. Doğrudan Ekran Alıntısı Modunu Başlatma
Menubar beklemeden anında ekran görüntüsü almak için:
```bash
cargo run --release -- --capture
```

### 4. Bir Görseli Masaüstüne Sabitleme (Pin)
```bash
cargo run --release -- --pin /path/to/image.png
```

---

## 🔐 macOS İzinleri
macOS'ta uygulamanın ekranı okuyabilmesi için **Ekran Kaydı (Screen Recording)** iznine ihtiyacı vardır:
1. **Sistem Ayarları (System Settings)** > **Gizlilik ve Güvenlik (Privacy & Security)** bölümüne gidin.
2. **Ekran Kaydı (Screen Recording)** sekmesinde terminalinize veya uygulamanıza izin verin.
