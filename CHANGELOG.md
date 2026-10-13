# 📝 Changelog - PhotoCraft Android

Todas as alterações notáveis deste projeto serão documentadas neste arquivo.

O formato segue as diretrizes do [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

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
