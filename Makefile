.PHONY: all app run install clean

all: app

# .app paketini derler ve oluşturur
app:
	@chmod +x scripts/bundle_app.sh
	@./scripts/bundle_app.sh

# Uygulamayı başlatır
run: app
	@open ScreenShot.app

# Uygulamayı /Applications dizinine taşır
install: app
	@echo "📂 /Applications/ScreenShot.app dizinine yükleniyor..."
	@rm -rf /Applications/ScreenShot.app
	@cp -R ScreenShot.app /Applications/
	@echo "✓ Başarıyla yüklendi! Spotlight veya Launchpad'den 'ScreenShot' olarak açabilirsiniz."

clean:
	@rm -rf target ScreenShot.app icon.iconset app_icon_1024.png
	@echo "✓ Temizlendi."
