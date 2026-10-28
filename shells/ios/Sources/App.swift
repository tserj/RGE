import UIKit
import WebKit

/// Нативная оболочка iOS: WKWebView с web-сборкой игры (WASM).
///
/// Сама игра одна для всех платформ: Bevy компилируется в WebAssembly
/// (см. корневой README, раздел «Web»), а оболочка лишь показывает её
/// на весь экран. Логика не дублируется.
final class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        let window = UIWindow(frame: UIScreen.main.bounds)

        let config = WKWebViewConfiguration()
        let web = WKWebView(frame: window.bounds, configuration: config)
        web.isOpaque = false
        web.backgroundColor = .black
        web.scrollView.isScrollEnabled = false // игра сама обрабатывает ввод

        // Релиз: web-сборка лежит в Resources/web (как её туда положить —
        // см. корневой README, раздел «iOS»).
        if let url = Bundle.main.url(forResource: "index", withExtension: "html", subdirectory: "web") {
            web.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent())
        }

        let root = UIViewController()
        root.view = web
        window.rootViewController = root
        window.makeKeyAndVisible()
        self.window = window
        return true
    }
}

@main
final class GameMenuApp {
    static func main() {
        UIApplicationMain(
            CommandLine.argc,
            CommandLine.unsafeArgv,
            nil,
            NSStringFromClass(AppDelegate.self)
        )
    }
}
