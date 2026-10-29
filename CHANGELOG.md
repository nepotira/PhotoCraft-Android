# 📝 Changelog - PhotoCraft Android

Todas as alterações notáveis deste projeto serão documentadas neste arquivo.

O formato segue as diretrizes do [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

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
