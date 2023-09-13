
# CountMeDown-rs
Countdowns for your streams.

![image](https://github.com/xenein/CountMeDown-rs/assets/76600392/a04eaef7-4189-43ed-84b8-ddc4cec42732)


Ein Nonxens-Projekt. Meine Tools sind frei und kostenlos verwendbar.\
Wer Geld dafür geben möchte, kann das über 
Ko-Fi einmalig oder regelmäßig tun.\
Wer dafür eine Rechnung braucht, kann eine mit ausgewiesenen Steuern bekommen.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A0P60CR)

## Wie? // How?

OBS kann Text aus Dateien lesen und im Stream anzeigen.\
OBS can read text from files and show read data in streams.

CountMeDown-rs schreibt dir einen Countdown in eine Datei, die du dann einbinden kannst.\
CountMeDown-rs writes countdowns to a file that you can use with OBS.

Du brauchst eine Zeit für den Countdown und einen Speicherpfad. Alles andere hat Standardwerte.\
You will need a time for the countdown and a path to save to. All else has standard values.

Zeit kann in Sekunden, in Minuten:Sekunden oder in Stunden:Minuten:Sekunden angegeben werden.\
Timme can be entered in Seconds in Minutes:Seconds or in Hours:Minutes:Seconds.

Es gibt keine Installation. Lade die Datei für dein System runter und starte sie.\
There is no setup. Download the file for your OS and start it.

Es könnte Warnungen geben, da der Code nicht signiert ist oder die Datei nicht als ausführbar markiert wurde.\
There can be warnings because the file is not code-signed and might not be marked as executable.

Auf macOS könnte das so aussehen // this could like this on macOS
```bash
sudo xattr -r -d com.apple.quarantine ./countmedown-rs-darwin-arm64
chmod +x countemdown-rs-darwin.arm64
```

### mehr? // advanced?

CountMeDown-rs kann auch auf einer Kommandozeile genutzt werden.\
CountMeDown-rs can be used on command line.

```bash
./countmedown-rs-darwin-arm64 --help
```

## Probleme? // Problems?

Wenn etwas nicht funktioniert, lege gern ein Issue dafür an.\
If something doesn't work, you can create an issue here. 

## Fortschritt // Progress

Das hier ist eine frühe Vorab-Version und weder vollständig noch stabil.\
This is an early prerelease and neither complete nor stable.

Es sollte Bugfixes und Verbesserungen geben.\
There will be fixes for bugs and overall improvements.

Geplante Funktionen beinhalten:\
Planned features include:

- Sound am Ende des Countdown abspielen // play sound when timer runs out
- Zieluhrzeit statt Ablaufzeit angeben // enter a target time instead of a duration
- Mehr Dokumentation // more documentation
- ein paar Tests wären nice // add some tests

## Selbst bauen // build yourself

CountMeDown-rs ist in rust mit den üblichen Werkzeugen gebaut.\
CountMeDown-rs is build in rust using the usual tools.

Du brauchst also cargo und eine Rust-Umgebgung. Mit rustup solltest du einfach eine besorgen können.\
You will need cargo and an environment to build rust. Using rustup you should be able to easily get one.

```bash
git clone git@github.com:xenein/CountMeDown-rs.git
cd CountMeDown-rs
cargo run
```
