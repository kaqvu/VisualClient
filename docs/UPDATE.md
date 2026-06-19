# Jak automatycznie aktualizować Visual Client (GitHub Actions)

Dzięki wbudowanemu systemowi wydawniczemu na GitHubie (GitHub Actions) oraz Tauri Updaterowi, aktualizacje dostarczają się automatycznie bezpośrednio do użytkowników! Poniżej instrukcja, jak wydać nową wersję.

## 1. Zmiana wersji
Musisz zaktualizować wersję aplikacji w trzech plikach na tę samą wartość (np. `26.1.1`):
1. `package.json` -> `"version": "26.1.1"`
2. `src-tauri/Cargo.toml` -> `version = "26.1.1"`
3. `src-tauri/tauri.conf.json` -> `"version": "26.1.1"`

## 2. Tworzenie wydania (Release) na GitHubie
Automatyczny system budowania uruchamia się tylko, gdy wypchniesz nowy Tag (etykietę) z prefixem `v`.

Z poziomu konsoli (lub aplikacji GitHub Desktop):

```bash
# 1. Dodaj i zapisz swoje zmiany z nową wersją
git add .
git commit -m "Release 26.1.1"

# 2. Utwórz nowy Tag wersji (musi zaczynać się od v, np. v26.1.1)
git tag v26.1.1

# 3. Wypchnij kod ORAZ Tagi do repozytorium
git push origin main
git push origin v26.1.1
```

## 3. Co dzieje się dalej?
- Po wpisaniu powyższych komend wejdź na swoje repozytorium GitHub do zakładki **Actions**.
- Zobaczysz, jak wirtualne serwery (Windows, macOS, Linux) kompilują w chmurze najnowszą wersję.
- Po zakończeniu (~15 minut) w zakładce **Releases** automatycznie pojawi się nowa paczka wraz z plikiem `latest.json`.
- Aplikacja automatycznie wykryje nowy numerek, pobierze aktualizację dla odpowiedniego systemu, a następnie poinformuje użytkownika o restarcie!

## WAŻNE: Klucze Updatera!
Aktualizator potrzebuje kluczy podpisu cyfrowego (by weryfikować paczki). Utworzyliśmy w tym celu publiczny klucz w `tauri.conf.json`, ale by akcje na serwerach GitHub miały dostęp do podpisywania, musisz ukryć klucz prywatny. 

Gdy wygenerujesz swoje docelowe klucze (`npm run tauri signer generate`), postępuj zgodnie z instrukcją podaną w konsoli i wrzuć tajne wartości do **GitHub Secrets**:
- Przejdź w repozytorium do `Settings > Secrets and variables > Actions > New repository secret`.
- Stwórz klucz `TAURI_SIGNING_PRIVATE_KEY` z wklejoną zawartością pliku klucza prywatnego (albo całą treść hasha).
- Jeśli ustawiłeś hasło, stwórz klucz `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
