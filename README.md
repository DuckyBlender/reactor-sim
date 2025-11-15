# reactor-sim

Mała gra symulująca sterowanie reaktorem zbudowana w [Bevy](https://bevyengine.org/). Twoim celem jest prowadzenie elektrowni jądrowej tak efektywnie, jak to możliwe, bez stopienia rdzenia reaktora lub zniszczenia turbiny.

## Rozgrywka

- Reguluj **reaktywność**, aby rozgrzać reaktor i wytworzyć parę.
- Reguluj przepływ **turbiny**, aby zamienić ciepło na moc i pieniądze.
- Uważaj na:
  - **Temperaturę i ciśnienie reaktora**
  - **Temperaturę i wytrzymałość turbiny**
  - **Wytworzoną moc i pieniądze**
  - **Poziom paliwa** (rozpada się w czasie z okresem półtrwania ~3 minuty)
- Jeśli temperatura reaktora lub turbiny przekroczy limit bezpieczeństwa, gra się kończy.
- Zacznij z **$1000** i zarabiaj pieniądze poprzez wytwarzanie mocy.

### Ulepszenia i konserwacja

- **Wymiana paliwa** ($250): Uzupełnij paliwo do 100%. Można użyć tylko gdy reaktywność i turbina są na 0%, a paliwo jest poniżej 90%.
- **Upgrade turbiny** ($500): Zwiększa maksymalny limit temperatury turbiny z 290°C do 350°C. Jednorazowy zakup.

Gra zawiera **menu główne** z opcjami rozpoczęcia nowej gry, ustawień, napisów końcowych i samouczka.

### Uranek (asystent operatora)

Asystent operatora **Uranek** znajduje się w lewym górnym rogu podczas rozgrywki:

- Miga bezczynnie podczas gry.
- Pokazuje dymek i odtwarza losowe dźwięki mówione, gdy:
  - Reaktor jest bliski przegrzania.
  - Turbina jest bliska przegrzania.
  - Zarabiasz dużo pieniędzy.
- Jego powitanie jest widoczne przez krótki czas na początku rozgrywki.

## Sterowanie

Klawiatura:

- `ESC` – Pauza / wznowienie gry.
- W menu: użyj myszy do nawigacji i klikania przycisków.

Interfejs:

- Użyj **suwaków**, aby ustawić docelową reaktywność i przepływ turbiny.
- Obserwuj **wskaźniki** po lewej stronie, aby utrzymać wszystkie wartości w bezpiecznych zakresach.
- Kliknij **przycisk wymiany paliwa** (prawy górny róg), aby uzupełnić paliwo, gdy warunki są spełnione.
- Kliknij **przycisk ulepszenia turbiny** (prawy górny róg), aby ulepszyć swoją turbinę.

## Kompilacja i uruchomienie

Wymagania:

- Rust (stabilna wersja) i Cargo zainstalowane.
- System zdolny do uruchomienia Bevy (Linux, macOS lub Windows z sterownikami graficznymi).

Sklonuj i uruchom:

```bash
cargo run --release
```

Zasoby znajdują się w katalogu `assets/` i są ładowane w czasie wykonywania (czcionki, sprite'y, modele i dźwięki). Upewnij się, że uruchamiasz binarkę z katalogu głównego projektu, aby względne ścieżki do zasobów były poprawnie rozwiązywane.

## Struktura projektu

- `src/`
  - `main.rs` – Punkt wejścia gry, konfiguracja aplikacji Bevy.
  - `simulation.rs` – Logika symulacji reaktora, turbiny i środowiska.
  - `sound.rs` – Muzyka w tle i ustawienia audio.
  - `tutorial.rs` – Przebieg samouczka.
  - `model.rs` – Model 3D reaktora.
  - `menu/` – System menu (menu główne, ustawienia, napisy końcowe).
    - `main_menu.rs`, `settings.rs`, `credits.rs`, itd.
  - `ui/` – Komponenty UI 2D (wskaźniki, suwaki, Uranek, koniec gry, pauza).
    - `indicators.rs`, `sliders.rs`, `uranek.rs`, `game_over.rs`, `pause.rs`, itd.
- `assets/`
  - `fonts/` – Czcionki UI.
  - `sprites/` – Sprite'y 2D (Uranek, grafika UI).
  - `imgs/` – Zasoby graficzne (ikony nuklearne, turbiny).
  - `sound/` – Muzyka w tle, efekty dźwiękowe (eksplozja, syczenie, upgrade) i klipy mówione Uranka (wiele w `talking/`).
  - `models/` – Model 3D reaktora.
- `examples/` – Przykładowe fragmenty kodu.

## Licencja

Ten projekt jest przeznaczony do celów hackathonowych / edukacyjnych. Dostosuj lub rozszerz go według potrzeb.
