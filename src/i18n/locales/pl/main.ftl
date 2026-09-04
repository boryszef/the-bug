### Polish strings — a first draft for a Polish speaker to review and
### correct (see docs/i18n-plan.md). Grammatical case is only resolved where
### `src/i18n/mod.rs` explicitly asks for it (terrain locative in
### `event-found`, item genitive in the "not enough X" events); other
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

## Item names (nominative), with a `.genitive` attribute for the "not
## enough X" events.

item-stick = Patyk
    .genitive = patyka
item-stone = Kamień
    .genitive = kamienia
item-vine = Pnącze
    .genitive = pnącza
item-cord = Sznurek
    .genitive = sznurka
item-stone-axe = Kamienny Topór
    .genitive = kamiennego topora
item-arrow = Strzała
    .genitive = strzały
item-wooden-bow = Drewniany Łuk
    .genitive = drewnianego łuku
item-plastic-bottle = Plastikowa Butelka
    .genitive = plastikowej butelki
item-copper-wire = Miedziany Drut
    .genitive = miedzianego drutu
item-coil = Cewka
    .genitive = cewki
item-pole = Drążek
    .genitive = drążka
item-microcontroller = Mikrokontroler
    .genitive = mikrokontrolera
item-speaker = Głośnik
    .genitive = głośnika
item-metal-detector = Wykrywacz Metalu
    .genitive = wykrywacza metalu
item-battery = Bateria
    .genitive = baterii
item-solar-panel = Panel Słoneczny
    .genitive = panelu słonecznego
item-solar-charger = Ładowarka Słoneczna
    .genitive = ładowarki słonecznej
item-circuit-board = Płytka Drukowana
    .genitive = płytki drukowanej
item-umbrella = Parasol
    .genitive = parasola
item-fabric = Tkanina
    .genitive = tkaniny

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
