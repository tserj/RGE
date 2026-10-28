# Сюда кладётся web-сборка игры (см. корневой README, раздел «iOS»):

trunk build --release
mkdir -p Resources/web
cp -r ../../dist/web/* Resources/web/

# Дальше — на macOS:
#   brew install xcodegen
#   cd shells/ios && xcodegen && open GameMenu.xcodeproj
