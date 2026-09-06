### Polish strings — a first draft for a Polish speaker to review and
### correct (see docs/i18n-plan.md). Grammatical case is only resolved where
### `src/i18n/mod.rs` explicitly asks for it (terrain locative in
### `event-found`, item genitive plural in the "not enough X" events, item
### accusative as the direct object of "Tworzysz"/"zrobić" in the crafting
### events); other interpolated nouns fall back to the nominative form.

## Event log lines.

event-awoke = Budzisz się i postanawiasz zbadać okolicę.
event-found = Znajdujesz { $item } { $terrain }.
event-found-poi = Znajdujesz { $item } { $poi }.
event-quest-accepted = Zadanie przyjęte: { $quest }
event-quest-completed = Zadanie ukończone: { $quest }!
event-unknown-recipe = Nie wiesz jak zrobić { $recipe }.
event-craft-shortage = Masz za mało { $needed }, aby zrobić { $output }.
event-craft-missing-tool = Potrzebujesz { $tool }, aby zrobić { $output }.
event-crafted = Tworzysz { $output }.
event-experiment-shortage = Eksperyment: { $items } -> za mało { $missing } (masz { $available }, potrzeba { $needed })
event-experiment-failed = Eksperyment: { $items } -> klapa
event-experimented = Eksperyment: { $items } -> { $output }{ $newly_learned ->
    [yes] { " (nowy przepis!)" }
   *[no] {""}
}
event-disassembled = Rozbierasz { $item }, odzyskując { $recovered }.
event-hunted = Polowanie: { $items }.
event-hunt-missed = Polowanie: zwierzyna uciekła.
event-hunt-unprepared = Potrzebujesz { $missing }, aby polować.

## Item names (nominative), with a `.genitive` attribute (singular), a
## `.genitive-plural` attribute — the latter is what the "not enough X"
## events use, since Polish wants the plural genitive for a shortage of a
## countable noun ("za mało kamieni", not "za mało kamień") — and an
## `.accusative` attribute (singular) for when the item is the direct
## object of a crafting verb ("Tworzysz Strzałę", not "Tworzysz Strzała").
## Where the nominative and accusative coincide (masculine/neuter nouns, and
## consonant-stem feminine ones like "gałąź"), the `.accusative` just repeats
## the name.

item-stick = Gałąź
    .genitive = gałęzi
    .genitive-plural = gałęzi
    .accusative = Gałąź
item-stone = Kamień
    .genitive = kamienia
    .genitive-plural = kamieni
    .accusative = Kamień
item-vine = Pnącze
    .genitive = pnącza
    .genitive-plural = pnączy
    .accusative = Pnącze
item-cord = Sznurek
    .genitive = sznurka
    .genitive-plural = sznurków
    .accusative = Sznurek
item-stone-axe = Kamienny Topór
    .genitive = kamiennego topora
    .genitive-plural = kamiennych toporów
    .accusative = Kamienny Topór
item-arrow = Strzała
    .genitive = strzały
    .genitive-plural = strzał
    .accusative = Strzałę
item-wooden-bow = Drewniany Łuk
    .genitive = drewnianego łuku
    .genitive-plural = drewnianych łuków
    .accusative = Drewniany Łuk
item-plastic-bottle = Plastikowa Butelka
    .genitive = plastikowej butelki
    .genitive-plural = plastikowych butelek
    .accusative = Plastikową Butelkę
item-copper-wire = Miedziany Drut
    .genitive = miedzianego drutu
    .genitive-plural = miedzianych drutów
    .accusative = Miedziany Drut
item-coil = Cewka
    .genitive = cewki
    .genitive-plural = cewek
    .accusative = Cewkę
item-pole = Drążek
    .genitive = drążka
    .genitive-plural = drążków
    .accusative = Drążek
item-microcontroller = Mikrokontroler
    .genitive = mikrokontrolera
    .genitive-plural = mikrokontrolerów
    .accusative = Mikrokontroler
item-speaker = Głośnik
    .genitive = głośnika
    .genitive-plural = głośników
    .accusative = Głośnik
item-metal-detector = Wykrywacz Metalu
    .genitive = wykrywacza metalu
    .genitive-plural = wykrywaczy metalu
    .accusative = Wykrywacz Metalu
item-battery = Bateria
    .genitive = baterii
    .genitive-plural = baterii
    .accusative = Baterię
item-solar-panel = Panel Słoneczny
    .genitive = panelu słonecznego
    .genitive-plural = paneli słonecznych
    .accusative = Panel Słoneczny
item-solar-charger = Ładowarka Słoneczna
    .genitive = ładowarki słonecznej
    .genitive-plural = ładowarek słonecznych
    .accusative = Ładowarkę Słoneczną
item-circuit-board = Płytka Drukowana
    .genitive = płytki drukowanej
    .genitive-plural = płytek drukowanych
    .accusative = Płytkę Drukowaną
item-umbrella = Parasol
    .genitive = parasola
    .genitive-plural = parasoli
    .accusative = Parasol
item-fabric = Tkanina
    .genitive = tkaniny
    .genitive-plural = tkanin
    .accusative = Tkaninę
item-electronic-toy = Elektroniczna Zabawka
    .genitive = elektronicznej zabawki
    .genitive-plural = elektronicznych zabawek
    .accusative = elektroniczną zabawkę
item-rusty-metal = Zardzewiały Metal
    .genitive = zardzewiałego metalu
    .genitive-plural = zardzewiałego metalu
    .accusative = zardzewniały metal
item-metal-knife = Metalowy Nóż
    .genitive = metalowego noża
    .genitive-plural = metalowych noży
    .accusative = metalowy nóż
item-electric-motor = Silnik Elektryczny
    .genitive = silnika elektrycznego
    .genitive-plural = silników elektrycznych
    .accusative = silnik elektryczny
item-steel-bolt = Stalowy Nit
    .genitive = stalowego nitu
    .genitive-plural = stalowych nitów
    .accusative = stalowy nit
item-meat = Mięso
    .genitive = mięsa
    .genitive-plural = mięsa
    .accusative = Mięso
item-bone = Kość
    .genitive = kości
    .genitive-plural = kości
    .accusative = Kość
item-hide = Skóra
    .genitive = skóry
    .genitive-plural = skór
    .accusative = Skórę
item-fur = Futro
    .genitive = futra
    .genitive-plural = futer
    .accusative = Futro

## Terrain names (nominative), with a `.locative` attribute (the full
## prepositional phrase, e.g. "w lesie" for "in the Forest") used by
## event-found instead of a separate glue word.

terrain-meadow = Łąka
    .locative = na łące
terrain-forest = Las
    .locative = w lesie
terrain-deadland = Pustkowie
    .locative = na pustkowiu

## Point-of-interest names (nominative), with the same `.locative` attribute.

poi-cave = Jaskinia
    .locative = w jaskini
poi-ruins = Ruiny
    .locative = w ruinach
poi-village = Osada
    .locative = w osadzie

## Quest name/description.

quest-craft-axe-name = Kłopoty na wschodzie
quest-craft-axe-description = Coś znów szykuje się na wschodzie. Od jakiegoś czasu było spokojnie, ale doświadczenie pokazuje, że kłopoty zawsze nadchodzą z tamtej strony. Cokolwiek to jest, lepiej się przygotuj! Idź do lasu, na łąkę i do jaskini, zbierz materiały, a potem poeksperymentuj z nimi i naucz się jak zrobić kamienną siekierę.
quest-explore-ruins-name = Cywilizacja Cyfrowa
quest-explore-ruins-description = Przejezdny podróżny wspomniał o pobliskich ruinach, podobno usianych starymi przedmiotami. Podobno na tych równinach znajdowało się niegdyś kilka wiosek - pozostałości wielkiej cywilizacji sprzed 500 lat — "Cywilizacji Cyfrowej", jak nazywają ją odkrywcy. Powinieneś zobaczyć to na własne oczy.
quest-stock-up-name = Zapasy na ciężkie czasy
quest-stock-up-description = Ktoś doświadczony w osadzie wciąż zerka na niebo. Idą chłody, mówi, a zapasy nie wystarczą dla wszystkich. Weź łuk, idź na łąki i do lasu i przynieś, co się da — mięso, skórę, kości, futro. Pięć porządnych polowań to dobry początek.

## Panel titles.

panel-map-title = Mapa
panel-player-title = Gracz
panel-events-title = Zdarzenia
panel-craft-title = Wytwórz
panel-recipes-title = Przepisy
panel-experiment-title = Eksperyment
panel-available-title = Dostępne
panel-selected-title = Wybrane
panel-disassemble-title = Rozmontuj
panel-items-title = Przedmioty
panel-quests-title = Zadania
panel-active-title = Aktywne

## Player panel.

player-level = Poziom: { $level }
player-xp = XP: { $xp }
player-recipes = Przepisy: { $known }/{ $total }
player-quests = Zadania: { $completed }/{ $total } — { $active }
player-quest-none = (brak)
player-inventory = Ekwipunek: { $items }
player-inventory-empty = (pusto)

## Craft panel.

craft-empty = Nie odkryłeś jeszcze żadnych przepisów.
craft-hint = ↑↓ ruch   Enter wytwórz   Esc anuluj

## Experiment panel.

experiment-hint = ↑↓ ruch   ←→ dodaj/usuń   Tab zmień kolumnę   e uruchom   Esc anuluj

## Disassemble panel.

disassemble-empty = Nie masz nic, co dałoby się rozłożyć.
disassemble-hint = ↑↓ ruch   Enter rozłóż   Esc anuluj

## Quests panel.

quests-active-none = Nie przyjęto żadnego zadania.
quests-active-blocked = Ukończ aktywne zadanie, aby przyjąć kolejne.
quests-available-empty = Brak dostępnych zadań.
quests-completed-none = Ukończone: (brak)
quests-completed = Ukończone: { $names }

## Footer key hints, one per panel.

footer-map = ←↑↓→ ruch   s szukaj   h poluj   [ ] panel   q wyjście
footer-experiment = ↑↓ ruch   ←→ dodaj/usuń   Tab zmień kolumnę   e uruchom   [ ] panel   q wyjście
footer-craft = ↑↓ ruch   Enter wytwórz   [ ] panel   q wyjście
footer-disassemble = ↑↓ ruch   Enter rozłóż   [ ] panel   q wyjście
footer-quests = ↑↓ ruch   Enter przyjmij   [ ] panel   q wyjście

## Sterowanie tylko w gui (pasek narzędzi, przyciski paneli, pasek podpowiedzi w src/gui/mod.rs).

action-quit = Wyjście (q)
action-experiment = Eksperymentuj
action-accept = Przyjmij
action-search = Szukaj (s)
action-hunt = Poluj (h)
gui-hint = [ ] zmiana kart   q wyjście
gui-hint-map = przeciągnij, aby przesunąć   przewiń, aby przybliżyć
