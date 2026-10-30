# 📝 Changelog - PhotoCraft Android

Todas as alterações notáveis deste projeto serão documentadas neste arquivo.

O formato segue as diretrizes do [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [0.1.7-android] - 2026-10-30 (Build 9)
*Base Upstream: PhotoCraft v0.17.0 (Sincronizado)*

### 🛠️ Correções de Erros & Aprimoramentos Críticos (Bug Fixes)
- **[BUG-028] / [FIX-028] Ativação Total de Atalhos de Teclado e Manipulação do Quadro Branco via Mouse (`Ctrl+Z`, `Espaço+Arrasto`, etc.)**:
  - *Sintoma*: Atalhos essenciais de teclado como `Ctrl+Z` (Desfazer), `Ctrl+Shift+Z` (Refazer), teclas de ferramentas (`V`, `M`, `L`, `B`, `H`, `Z`), `[` e `]` (tamanho do pincel) e a movimentação do quadro branco via mouse com a tecla `Espaço` mantida pressionada (ferramenta Mão temporária) não funcionavam no Googlebook OS.
  - *Causa Raiz*: Em `crates/ui-egui/src/app/frame.rs:84`, a chamada `shortcuts::handle(self, ctx)` estava encapsulada sob `if type_tool::in_ui_pass(ctx)`. No ciclo de vida do eframe, o método `logic` é chamado antes da passagem de interface (`ctx.run`), fazendo com que `in_ui_pass(ctx)` retornasse `false` durante o `logic` e silenciasse todos os atalhos de teclado.
  - *Solução*: (a) Inserido despacho explícito de `photocraft_ui_egui::shortcuts::handle(app, ui.ctx())` diretamente na avaliação de interface no `AndroidShell::ui`, onde todos os eventos do frame atual estão ativos e validados; (b) Chamada de `wheel_nav::fold_legacy_pinch` habilitada no `raw_input_hook` para Android/Googlebook OS, permitindo pinça e rolagem fluida de touchpad; (c) Desbloqueada a navegação do quadro branco ao manter a barra de espaço pressionada e arrastar com o botão esquerdo do mouse, com suporte nativo a arrasto pelo botão do meio do mouse e atalho `H`.
- **[BUG-029] / [FIX-029] Leitura Assíncrona e Não-Bloqueante de Arquivos do Google Drive (Fim dos Travamentos de I/O)**:
  - *Sintoma*: Ao soltar um arquivo arrastado do Google Drive no PhotoCraft, o aplicativo congelava a tela ("travando ao tentar abrir arquivo do Drive arrastado").
  - *Causa Raiz*: (1) A leitura do `ContentResolver.openInputStream()` (`read_content_or_file_uri`) era executada de forma síncrona na thread principal de renderização; como o provedor do Google Drive transfere os dados via rede e IPC, o loop gráfico ficava paralisado até a conclusão do download; (2) Ocorria colisão de I/O por leitura duplicada, pois tanto o `onDrop` do Java quanto o `poll_drop_helper` abriam a mesma URI em paralelo.
  - *Solução*: (a) Toda a ingestão de URIs de ContentProvider e Intents foi desacoplada em threads secundárias dedicadas (`std::thread::spawn(move || { ... enqueue_pending_file(...) })`); (b) No `DropHelper.java`, a entrega de eventos `ACTION_DROP` foi separada em worker threads assíncronas; (c) A interface permanece a 60/120 FPS sem nenhuma interrupção enquanto arquivos pesados ou remotos são baixados e carregados.
- **[BUG-030] / [FIX-030] Eliminação de Overhead no Loop Gráfico e Ativação de Background Jobs**:
  - *Sintoma*: Queda de fluidez e lentidão no editor ("photocraft lento em desempenho. Desempenho meio porco.").
  - *Causa Raiz*: (1) `poll_drop_helper()` executava `vm.attach_current_thread()` e reflexão JNI a cada frame (60–120x por segundo), provocando pressão sobre o garbage collector do ART e contenção de locks; (2) `app.background_jobs` permanecia como `false`, desativando threads de renderização em segundo plano (`background_redraw`) e empacando o pipeline na UI thread.
  - *Solução*: (a) Ativado `app.background_jobs = true;`, liberando a renderização assíncrona do canvas e filtros em núcleos paralelos; (b) Criada flag estática volátil `hasPendingDrops` no Java e amostragem espaçada no Rust (a cada 10 frames), zerando chamadas JNI desnecessárias.
- **[BUG-031] / [FIX-031] Correção Definitiva do Cursor do Mouse (Hover) na Árvore de Views do Android 17 / Googlebook OS**:
  - *Sintoma*: O cursor do mouse permanecia na seta padrão mesmo ao passar sobre elementos com cursor personalizado (texto, mão, cruz, redimensionadores).
  - *Causa Raiz*: O `NativeActivity` hospeda uma `SurfaceView` como filha da árvore de views. O método `setPointerIcon()` chamado apenas na raiz não era herdado pela `SurfaceView`, cujo método `onResolvePointerIcon()` retornava nulo.
  - *Solução*: (a) Criada a função recursiva `applyPointerIconRecursively()` em `DropHelper.java`, descendo por toda a hierarquia de `ViewGroup`s e atribuindo o ponteiro a todas as views e `SurfaceView`s; (b) Registro de `OnHoverListener` na raiz com o ponteiro ativo salvo em `sCurrentPointerIcon`, aplicando dinamicamente o cursor correto em eventos `ACTION_HOVER_MOVE` e `ACTION_HOVER_ENTER`; (c) Forçada a apropriação de foco de teclado físico na janela (`decorView.requestFocus()`).

---

## [0.1.6-android] - 2026-10-30 (Build 8)
*Base Upstream: PhotoCraft v0.17.0 (Sincronizado)*

### 🛠️ Correções de Erros & Aprimoramentos Críticos (Bug Fixes)
- **[BUG-024] / [FIX-024] Correção do Erro 'no importer configured' ao Abrir Arquivos PSD**:
  - *Sintoma*: Ao clicar em arquivos como `3mam11_black_golden.psd` ou abrir documentos no Android/Googlebook OS, o app reportava a falha `no importer configured` e impedia o carregamento.
  - *Causa Raiz*: `apps/photocraft-android/src/lib.rs` inicializava `Services::default()` sem configurar `services.import`, que permanecia `None`. Em `crates/ui-egui/src/app/files.rs:71`, `open_bytes_from` exige `self.services.import.as_ref().ok_or("no importer configured")?`.
  - *Solução*: Conectados os serviços essenciais de decodificação e exportação do motor Photocraft (`services.import` via `photocraft_io::import_with_options`, `services.export` via `photocraft_io::export`, `services.save_file`, `services.write`, `services.file_stamp` e `services.same_file`). Arquivos `.psd` abrem com suas camadas, máscaras e modos de mesclagem perfeitamente renderizados.
- **[BUG-025] / [FIX-025] Injeção de classes.dex no ClassLoader Primário e Correção do Hover do Mouse**:
  - *Sintoma*: O cursor do mouse não mudava para mão, texto, cruz ou redimensionamento ao passar sobre os elementos da interface no Googlebook OS.
  - *Causa Raiz*: (1) A classe `DropHelper` carregada via `InMemoryDexClassLoader` não podia ser resolvida por `env.find_class()` durante o loop de eventos na thread Rust; (2) O fallback tentava alterar o cursor invocando diretamente `decorView.setPointerIcon` fora da thread de interface, disparando `CalledFromWrongThreadException`.
  - *Solução*: (a) Salva referência global JNI (`DROP_HELPER_CLASS`) garantindo acesso universal de qualquer thread; (b) No CI `.github/workflows/android.yml`, `classes.dex` é injetado diretamente na raiz do APK e reassinado com `apksigner` e a `release.keystore`, integrando-o ao `PathClassLoader` nativo do Android; (c) Despacho estrito de `setPointerIcon` via `activity.runOnUiThread()` cobrindo `decorView` e `contentView`; (d) Atualização de cursor posicionada imediatamente após a avaliação da UI no `AndroidShell::ui`.
- **[BUG-026] / [FIX-026] Drag & Drop Failsafe para Google Drive e Downloads**:
  - *Sintoma*: O arrastar de arquivos do Google Drive ou da pasta local Downloads para dentro do PhotoCraft não abria os arquivos.
  - *Causa Raiz*: O binding JNI nativo para `onDrop` falhava silenciosamente entre classloaders isolados e as permissões de URI concedidas por `activity.requestDragAndDropPermissions(event)` eram prematuramente coletadas pelo GC antes da leitura dos dados pelo `ContentResolver`.
  - *Solução*: (a) Criada fila estática thread-safe `PENDING_DROPS` em Java (`queueDrop` e `pollDrop`), possibilitando polling ativo e seguro a cada ciclo de renderização no Rust; (b) Retenção estática de `sLastPermissions` mantendo os direitos de acesso ao Content Provider concedidos pelo sistema ativos durante a leitura; (c) Anexação dupla de listeners de drag na raiz da janela e no container de conteúdo; (d) Suporte unificado para extração de URIs e dados textuais.
- **[BUG-027] / [FIX-027] Intent Filters Abrangentes para Arquivos PSD**:
  - *Solução*: Expandidos os intent filters em `Cargo.toml` adicionando blocos específicos para `application/octet-stream` e `*/*` com padrões de extensão `.psd` e `.PSD`, assegurando reconhecimento irrestrito do PhotoCraft pelo seletor do sistema no Googlebook OS.

---

## [0.1.5-android] - 2026-10-30 (Build 7)
*Base Upstream: PhotoCraft v0.17.0 (Sincronizado)*

### ✨ Recursos & Otimizações Desktop (Googlebook OS & Samsung DeX)
- **[FEAT-003] Suporte a Drag & Drop Global (Google Drive & Downloads)**:
  - Implementado listener nativo `OnDragListener` acoplado via `InMemoryDexClassLoader` com concessão de permissão sob demanda via `activity.requestDragAndDropPermissions(event)`. Permite arrastar arquivos `.psd`, imagens e outros formatos diretamente do Google Drive ou da pasta Downloads para dentro do PhotoCraft.
- **[FEAT-004] Mudança Dinâmica do Cursor do Mouse (Hover)**:
  - Conexão em tempo real entre o `egui::CursorIcon` e a API `android.view.PointerIcon` do Android 7.0+ / GooglebookOS, com despacho direto na thread UI (`TYPE_ARROW`, `TYPE_HAND`, `TYPE_TEXT`, `TYPE_CROSSHAIR`, `TYPE_HORIZONTAL_DOUBLE_ARROW`, `TYPE_VERTICAL_DOUBLE_ARROW`, `TYPE_GRAB`, `TYPE_GRABBING`, etc.).
- **[FEAT-005] Associação de Tipos de Arquivo `.psd`**:
  - Declarados intent filters para ações `VIEW`, `EDIT` e `SEND` nos esquemas `content` e `file` com tipos MIME `image/vnd.adobe.photoshop`, `image/x-photoshop`, `application/x-photoshop`, `image/psd`. Clicar em qualquer arquivo `.psd` no sistema agora abre o PhotoCraft e carrega o documento automaticamente.
- **[FEAT-006] Compensação Inteligente da Barra de Título do Googlebook OS**:
  - Substituição de `Theme.NoTitleBar.Fullscreen` por `Theme.DeviceDefault.NoActionBar`, leitura de `WindowInsets.Type.captionBar()` e adição de espaçador superior responsivo com botões de ajuste fino `[Barra: ➖ XXpt ➕]`, eliminando sobreposições dos menus pelas decorações da janela.
- **[SYNC-001] Sincronização Obrigatória com Base Upstream (`storytold/photocraft`)**:
  - Incorporadas 5.393 atualizações upstream (versão base `v0.17.0`), preservando todas as adaptações mobile (gestos de pinça multitoque no canvas, layout responsivo de diálogos e runner Android).

### 🛠️ Correções de Erros (Bug Fixes)
- **[BUG-018] / [FIX-018] Correção do Hover do Mouse**: Ícone do ponteiro agora atualiza dinamicamente sobre botões, canvas, réguas e divisores.
- **[BUG-019] / [FIX-019] Correção do Ícone Oficial no Launcher**: Metadados `icon = "@mipmap/ic_launcher"` adicionados nas tags `application` e `activity` do `Cargo.toml`.
- **[BUG-020] / [FIX-020] Abertura de PSD via Intent**: Captura de URIs em `getIntent()` e decodificação via `ContentResolver`.
- **[BUG-021] / [FIX-021] Drag & Drop do Google Drive**: Permissões de leitura cruzada desbloqueadas via `requestDragAndDropPermissions`.
- **[BUG-022] / [FIX-022] Drag & Drop de Downloads**: Acesso e ingestão automática de arquivos locais arrastados.
- **[BUG-023] / [FIX-023] Fim da Sobreposição da Barra Superior**: Menus do PhotoCraft iniciam perfeitamente abaixo da barra de título do Googlebook OS.

---

## [0.1.4-android] - 2026-10-29 (Build 6)
*Base Upstream: PhotoCraft v0.17.0*

### ✨ Recursos & Otimizações Multi-Plataforma
- **[FEAT-002] Ícone Oficial do PhotoCraft Integrado**:
  - *Objetivo*: Substituir o ícone genérico do sistema pelo logo oficial do PhotoCraft.
  - *Solução*: Incorporados os assets oficiais em alta definição de `assets/app-icon/` nas resoluções `mdpi` (48x48), `hdpi` (64x64), `xhdpi` (128x128), `xxhdpi` (256x256), `xxxhdpi` (512x512) e `drawable/ic_launcher.png`, configurados via `[package.metadata.android]` com `icon = "@mipmap/ic_launcher"` e `resources = "res"`.
- **[FIX-017] Suporte Completo a GPUs Intel UHD (Chromebooks Jinlon / Googlebook OS)**:
  - *Causa*: No ChromeOS ARCVM e em dispositivos Intel Comet Lake (placa Jinlon), o Vulkan não é exposto por padrão aos containers Android, oferecendo aceleração total via OpenGL ES 3.2. A configuração padrão do WGPU em Android restringia-se ao Vulkan, causando falha e fechamento prematuro do renderizador.
  - *Solução*: Configurado pipeline dual de renderização em `lib.rs` com `Backends::VULKAN | Backends::GL` (OpenGL ES via EGL), selecionando preferência de alta performance (`HighPerformance`), aplicando os limites reais do adaptador gráfico via `use_adapter_limits` e desativando `VALIDATION_INDIRECT_CALL` para evitar falhas do compilador de shaders em GPUs Intel integradas.
- **[RULE-007] Mandato Universal de Limpeza de APKs Anteriores no Computador**:
  - *Objetivo*: Manter o ambiente limpo e evitar confusão na instalação manual.
  - *Solução*: Regra obrigatória inserida em `.agents/rules/universal-engineering-rules.md` e `GEMINI.md` determinando que, a cada nova versão gerada e baixada para `dist/`, todos os APKs de versões legadas devem ser imediatamente apagados.

### 🛠️ Correções de Falhas Conhecidas (Bug Fixes)
- **[FIX-016] Eliminação de Crash na Inicialização (Tela Branca e Fechamento Instantâneo)**:
  - *Sintoma*: O usuário relatou que o app abria rapidamente, exibia uma tela branca rápida e fechava em loop.
  - *Causa Raiz Técnica*: (1) A função `set_immersive_sticky()` era chamada na thread de background de `android_main` antes do `ANativeWindow` estar inicializado e anexado pelo sistema, lançando `CalledFromWrongThreadException` na camada Java do Android; (2) O JNI não limpava a exceção pendente (`exception_clear`), fazendo com que o runtime ART da JVM abortasse o processo via `SIGABRT` no próximo comando JNI.
  - *Solução*: (a) Removida a chamada prematura de `set_immersive_sticky()` de antes do `eframe::run_native`; (b) Criado o helper inline `clear_jni_exception` e adicionada limpeza estrita de exceções em todas as operações JNI (`has_storage_permission`, `request_android_permissions`, `open_app_storage_settings`, `set_immersive_sticky`); (c) Registrado hook global de pânico com `std::panic::set_hook` para capturar e registrar qualquer anomalia no Android logcat.

---

## [0.1.3-android] - 2026-10-29 (Build 5)
*Base Upstream: PhotoCraft v0.17.0*

### ✨ Recursos & Otimizações Multi-Plataforma
- **[FEAT-001] Suporte Dual-ABI Nativo (Android ARM64 & Googlebook OS / Chromebook x86_64)**:
  - *Objetivo*: Eliminar o gargalo de tradução binária (Houdini/NDK) em Chromebooks e PCs x86_64 com ChromeOS / Googlebook OS.
  - *Solução*: Configurado suporte nativo para `aarch64-linux-android` e `x86_64-linux-android` no `Cargo.toml` e no workflow de CI `android.yml`. O app roda 100% nativo com aceleração gráfica WGPU em ambas as plataformas sem perda de performance.

### 🛠️ Correções de Erros (Bug Fixes)
- **[FIX-012] Filtro Dinâmico de Extensões no Seletor de Arquivos**:
  - *Causa*: O seletor listava todos os arquivos indistintamente, dificultando achar imagens e PSDs em pastas cheias.
  - *Solução*: Implementada filtragem dinâmica baseada em `request.filters` (`.psd`, `.png`, `.jpg`, etc.) com caixa de seleção "Exibir todos os formatos" para alternância sob demanda.
- **[FIX-013] Resolução Robusta de Armazenamento Android Multi-Usuário**:
  - *Causa*: Caminhos rígidos para `/sdcard` falhavam em ambientes com perfis de trabalho ou caminhos modernos `/storage/emulated/0` ou `/storage/emulated/10`.
  - *Solução*: Implementada detecção automática do root de armazenamento real do sistema, adicionando atalhos diretos e confiáveis.
- **[FIX-014] Zona Morta (Deadzone) nos Gestos de Pinça no Canvas**:
  - *Causa*: Variações infinitesimais no toque de 2 dedos geravam micro-tremores (jitter) indesejados ao segurar a tela.
  - *Solução*: Adicionado limiar de tolerância de 0.005 para zoom e rotação, tornando a navegação suave e estável.
- **[FIX-015] Rastreamento Mandatório Contínuo do Upstream PhotoCraft**:
  - *Causa*: Ausência de regra rígida para incorporar novas versões da storytold.
  - *Solução*: Estabelecido o arquivo de persistência `.agents/rules/upstream-synchronization-rules.md` obrigando checagem e merge contínuo do upstream a cada ciclo de desenvolvimento.

---

## [0.1.2-android] - 2026-10-13 (Build 4)
*Base Upstream: PhotoCraft v0.8.2*

### 🛠️ Correções de Erros (Bug Fixes)
- **[FIX-005] Glifos e Símbolos Visíveis (Fim dos Quadrados `□`)**:
  - *Causa*: O egui não possui emojis ou setas Unicode (`➔`) na sua fonte padrão embutida, renderizando caixas de caracteres desconhecidos.
  - *Solução*: Substituídos símbolos incompatíveis por caracteres ASCII universais e limpos (`>`, `<`), garantindo 100% de legibilidade em qualquer dispositivo.
- **[FIX-006] Confirmação de Permissões em Tempo Real**:
  - *Causa*: O botão ficava verde logo após a solicitação, sem validar se o usuário de fato autorizou o acesso no sistema operacional.
  - *Solução*: Implementada checagem contínua via JNI (`has_storage_permission`). O botão exibe estado vermelho/pendente caso não autorizado e só se torna verde após confirmação do Android, com instruções claras e atalho para as configurações manuais.
- **[FIX-007] Modo Imersivo Sticky Real (Ocultação da Barra Superior)**:
  - *Causa*: O tema antigo do manifesto não forçava o desaparecimento das barras de sistema no Android 11+.
  - *Solução*: Invocado `set_immersive_sticky` via JNI configurando as flags `SYSTEM_UI_FLAG_IMMERSIVE_STICKY`, `LAYOUT_HIDE_NAV` e `FULLSCREEN` no DecorView. As barras do sistema ficam 100% ocultas.
- **[FIX-008] Diálogo "Novo Documento" Totalmente Responsivo**:
  - *Causa*: O diálogo continha largura mínima rígida de 800pt e barra de presets de 520pt sem rolagem, vazando completamente da tela em celulares.
  - *Solução*: Largura mínima e máxima adaptadas dinamicamente à largura da tela do aparelho e conteúdo envelopado em `egui::ScrollArea::both()`, permitindo rolagem com o dedo para visualizar todos os campos e criar documentos confortavelmente.
- **[FIX-009] Gestos Multitoque na Área de Desenho (Estilo IbisPaint X) & Rolagem com Toque**:
  - *Causa*: Ao encostar dois dedos na tela para dar zoom ou mover, o primeiro dedo era interpretado pelo PhotoCraft como pincel, traçando riscos acidentais.
  - *Solução*: Ao detectar 2 dedos (`num_touches >= 2`), os traços da ferramenta ativa são cancelados e o aplicativo entra em modo de navegação com pinça de zoom, translação e rotação livres. Com 1 dedo, o desenho funciona normalmente. Também ativado `drag_to_scroll = true` em todas as listas e menus.
- **[FIX-010] Seletor de Arquivos Integrado para Android**:
  - *Causa*: Clicar em "Arquivo > Abrir" ou "Exportar Como" não respondia por falta de backend de diálogo de arquivos para Android.
  - *Solução*: Criado o módulo `file_picker.rs`, que abre uma janela modal com atalhos para `/sdcard/Download`, `/sdcard/Pictures`, `/sdcard/DCIM` e permite abrir e salvar arquivos no armazenamento do celular sem travar a interface.

---

## [0.1.1-android] - 2026-10-13 (Build 3)
*Base Upstream: PhotoCraft v0.8.2*

### 🛠️ Correções de Erros (Bug Fixes)
- **[FIX-001] Assinatura Criptográfica Fixa & Atualizações sem Desinstalar**:
  - *Causa*: O workflow gerava uma nova keystore aleatória a cada build do CI, mudando o certificado e fazendo o Android rejeitar atualizações ("Como o pacote parece ser inválido, o app não foi instalado").
  - *Solução*: Keystore RSA persistente (`release.keystore`) incluída no repositório com `version_code = 3`. Atualizações agora instalam normalmente por cima da versão existente.
- **[FIX-002] Chamada Nativa de Permissões Android via JNI**:
  - *Causa*: O assistente anterior apenas alternava uma flag booleana em memória ao clicar no botão, sem invocar o sistema operacional.
  - *Solução*: Implementada ponte JNI nativa em Rust (`request_android_permissions`) invocando `Activity.requestPermissions()` para acionar a caixa de diálogo nativa do Android na tela.
- **[FIX-003] Fim do Stuttering/Flicker e Correção de DPI/Escala**:
  - *Causa*: A chamada a `ctx.set_pixels_per_point(0.85)` a cada frame forçava uma densidade arbitrária em conflito com o `winit`, gerando alternância rápida (piscadas) entre duas escalas e tornando os elementos microscópicos em telas de alta densidade.
  - *Solução*: Removida a sobreposição forçada de `pixels_per_point`. Adoção de `ctx.set_zoom_factor(factor)` que respeita o DPI nativo da tela e redimensiona suavemente widgets e fontes sem piscar.
- **[FIX-004] Modo Tela Cheia Imersivo (Fullscreen)**:
  - *Causa*: A barra de status com relógio, sinal e bateria ficava sobreposta na área superior de trabalho do aplicativo.
  - *Solução*: Ativadas as flags `fullscreen = true` e o tema `@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen` no manifesto, ocultando barras de sistema para uma experiência imersiva completa.

### 🎨 Melhorias de Interface
- O card do assistente inicial agora é totalmente responsivo: ocupa a largura ideal da tela do smartphone e se centraliza em tablets e Samsung DeX sem encolher nas extremidades.

---

## [0.1.0-android] - 2026-10-13 (Build 2)
*Base Upstream: PhotoCraft v0.8.2*

### ✨ Adicionado
- Assistente inicial de Boas-Vindas e Onboarding estilo emulador de Nintendo Switch (Skyline/Yuzu).
- Declaração de permissões de armazenamento (`READ_EXTERNAL_STORAGE`, `WRITE_EXTERNAL_STORAGE`, `READ_MEDIA_IMAGES`, `MANAGE_EXTERNAL_STORAGE`, `POST_NOTIFICATIONS`) no `AndroidManifest.xml`.
- Suporte a multi-janela e redimensionamento livre para Samsung DeX e AluminiumOS (`resizeable_activity = true`).
- Barra de controle superior com botões de zoom `➖` e `➕` e atalho para o assistente.

### ⚠️ Erros Identificados e Relatados
- `[BUG-001]`: Impossibilidade de atualizar sobre a versão anterior sem desinstalar (certificado não fixo no CI).
- `[BUG-002]`: Botão de conceder permissões não disparava o diálogo do sistema Android.
- `[BUG-003]`: Interface com elementos minúsculos e piscando/travando em transição contínua de escala.
- `[BUG-004]`: Barra de notificações do sistema sobrepondo os menus e botões superiores.

---

## [0.0.1-android] - 2026-10-13 (Build 1)
*Base Upstream: PhotoCraft v0.8.2*

### ✨ Adicionado
- Prova de Conceito (PoC) inicial de compilação do PhotoCraft (Rust 2024 / wgpu / eframe 0.36) para arquitetura `aarch64-linux-android`.
- Geração do primeiro APK funcional abrindo a interface completa do desktop no Android.
- CI/CD automatizado via GitHub Actions.

### ⚠️ Limitações Conhecidas
- Acesso ao armazenamento não solicitado na inicialização.
- Interface desktop sem barra de compensação para telas compactas.
