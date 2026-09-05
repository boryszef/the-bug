### Polish strings — a first draft for a Polish speaker to review and
### correct (see docs/i18n-plan.md). Grammatical case is only resolved where
### `src/i18n/mod.rs` explicitly asks for it (terrain locative in
### `event-found`, item genitive plural in the "not enough X" events); other
### interpolated nouns fall back to the nominative form.

## Event log lines.

event-awoke = Budzisz się i postanawiasz zbadać okolicę.
event-found = Znajdujesz { $item } { $terrain }.
event-quest-accepted = Zadanie przyjęte: { $quest }
event-quest-completed = Zadanie ukończone: { $quest }!
event-unknown-recipe = Nie wiesz jak zrobić { $recipe }.
event-craft-shortage = Masz za mało { $needed }, aby zrobić { $output }.
event-crafted = Tworzysz { $output }.
event-experiment-shortage = Eksperyment: { $items } → za mało { $missing } (masz { $available }, potrzeba { $needed })
event-experiment-failed = Eksperyment: { $items } → klapa
event-experimented = Eksperyment: { $items } → { $output }{ $newly_learned ->
    [yes] { " (nowy przepis!)" }
   *[no] {""}
}
event-disassembled = Rozbierasz { $item }, odzyskując { $recovered }.

## Item names (nominative), with a `.genitive` attribute (singular) and a
## `.genitive-plural` attribute — the latter is what the "not enough X"
## events use, since Polish wants the plural genitive for a shortage of a
## countable noun ("za mało patyków", not "za mało patyka").

item-stick = Patyk
    .genitive = patyka
    .genitive-plural = patyków
item-stone = Kamień
    .genitive = kamienia
    .genitive-plural = kamieni
item-vine = Pnącze
    .genitive = pnącza
    .genitive-plural = pnączy
item-cord = Sznurek
    .genitive = sznurka
    .genitive-plural = sznurków
item-stone-axe = Kamienny Topór
    .genitive = kamiennego topora
    .genitive-plural = kamiennych toporów
item-arrow = Strzała
    .genitive = strzały
    .genitive-plural = strzał
item-wooden-bow = Drewniany Łuk
    .genitive = drewnianego łuku
    .genitive-plural = drewnianych łuków
item-plastic-bottle = Plastikowa Butelka
    .genitive = plastikowej butelki
    .genitive-plural = plastikowych butelek
item-copper-wire = Miedziany Drut
    .genitive = miedzianego drutu
    .genitive-plural = miedzianych drutów
item-coil = Cewka
    .genitive = cewki
    .genitive-plural = cewek
item-pole = Drążek
    .genitive = drążka
    .genitive-plural = drążków
item-microcontroller = Mikrokontroler
    .genitive = mikrokontrolera
    .genitive-plural = mikrokontrolerów
item-speaker = Głośnik
    .genitive = głośnika
    .genitive-plural = głośników
item-metal-detector = Wykrywacz Metalu
    .genitive = wykrywacza metalu
    .genitive-plural = wykrywaczy metalu
item-battery = Bateria
    .genitive = baterii
    .genitive-plural = baterii
item-solar-panel = Panel Słoneczny
    .genitive = panelu słonecznego
    .genitive-plural = paneli słonecznych
item-solar-charger = Ładowarka Słoneczna
    .genitive = ładowarki słonecznej
    .genitive-plural = ładowarek słonecznych
item-circuit-board = Płytka Drukowana
    .genitive = płytki drukowanej
    .genitive-plural = płytek drukowanych
item-umbrella = Parasol
    .genitive = parasola
    .genitive-plural = parasoli
item-fabric = Tkanina
    .genitive = tkaniny
    .genitive-plural = tkanin

## Terrain names (nominative), with a `.locative` attribute (the full
## prepositional phrase, e.g. "w lesie" for "in the Forest") used by
## event-found instead of a separate glue word.

terrain-meadow = Łąka
    .locative = na łące
terrain-forest = Las
    .locative = w lesie
terrain-cave = Jaskinia
    .locative = w jaskini
terrain-ruins = Ruiny
    .locative = w ruinach
terrain-village = Wieś
    .locative = we wsi
terrain-deadland = Pustkowie
    .locative = na pustkowiu

## Quest name/description.

quest-craft-arrows-name = Wyprodukuj Strzały
quest-craft-arrows-description = Coś znów szykuje się na wschodzie. Od jakiegoś czasu było spokojnie, ale doświadczenie pokazuje, że kłopoty zawsze nadchodzą z tamtej strony. Cokolwiek to jest, spłoszyło grubszą zwierzynę, więc grupa miejscowych myśliwych szykuje się na polowanie. Poprosili cię o przygotowanie 5 strzał. Idź do lasu po patyki, a potem poeksperymentuj z nimi, by nauczyć się, jak robi się strzały.
quest-explore-ruins-name = Zbadaj Ruiny
quest-explore-ruins-description = Przejezdny podróżny wspomniał o pobliskich ruinach, podobno usianych starymi przedmiotami. Podobno na tych równinach znajdowało się niegdyś kilka wiosek - pozostałości wielkiej cywilizacji sprzed 500 lat — "Cywilizacji Cyfrowej", jak nazywają ją odkrywcy. Powinieneś zobaczyć to na własne oczy.

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
quests-completed-none = Ukończone: (jeszcze żadnych)
quests-completed = Ukończone: { $names }

## Footer key hints, one per panel.

footer-map = ←↑↓→ ruch   s szukaj   [ ] panel   q wyjście
footer-experiment = ↑↓ ruch   ←→ dodaj/usuń   Tab zmień kolumnę   e uruchom   [ ] panel   q wyjście
footer-craft = ↑↓ ruch   Enter wytwórz   [ ] panel   q wyjście
footer-disassemble = ↑↓ ruch   Enter rozłóż   [ ] panel   q wyjście
footer-quests = ↑↓ ruch   Enter przyjmij   [ ] panel   q wyjście
