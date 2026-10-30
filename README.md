<p align="center">
  <img alt="PhotoCraft Android" src="docs/brand/artcraft-logo.svg" width="180">
</p>

<p align="center">
  <img src="assets/app-icon/hicolor/256x256/apps/ai.storyteller.photocraft.png" width="128" height="128" alt="PhotoCraft icon: an engraved nine-tailed fox on a blue tile">
</p>

<h1 align="center">PhotoCraft Android & Googlebook OS</h1>

<p align="center">
  <b>A primeira edição para Android do PhotoCraft — O Photoshop de código aberto em Rust.</b><br>
  Camadas, máscaras, ajustes, vetores, pincéis e arquivos PSD reais no seu celular, tablet e Samsung DeX.<br>
  <i>Port comunitário oficial mantido e adaptado por <b>Nepo</b> (<a href="https://github.com/nepotira">@nepotira</a>).</i>
</p>

<p align="center">
  <a href="https://github.com/nepotira/PhotoCraft-Android/releases"><img alt="Android Version" src="https://img.shields.io/badge/Android%20Edition-v0.1.6--android-34A853?style=for-the-badge&logo=android&logoColor=white"></a>
  <a href="https://github.com/storytold/photocraft"><img alt="Upstream Base" src="https://img.shields.io/badge/Upstream%20Base-PhotoCraft%20v0.17.0-FF9900?style=for-the-badge&logo=github"></a>
  <img alt="100% Rust" src="https://img.shields.io/badge/100%25-Rust-b7410e?style=for-the-badge&logo=rust">
  <img alt="Samsung DeX Ready" src="https://img.shields.io/badge/Samsung%20DeX-Compat%C3%ADvel-1428A0?style=for-the-badge">
  <img alt="Googlebook OS Ready" src="https://img.shields.io/badge/Googlebook%20OS%20(x86)-Nativo-4285F4?style=for-the-badge">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=for-the-badge">
</p>

---

## 📌 Sobre este Repositório (Fork Android por Nepo)

Este repositório é um **fork de código aberto** do projeto original [storytold/photocraft](https://github.com/storytold/photocraft) (suíte **ArtCraft** criada por Brandon Thomas).

O objetivo deste projeto, liderado por **Nepo**, é viabilizar e otimizar a experiência do PhotoCraft para smartphones, tablets e ambientes desktop Android (**Samsung DeX**, Chromebooks / Googlebook OS e o futuro **AluminiumOS**).

---

## ✨ Recursos Especiais da Versão Android

* 🎨 **Ícone Oficial do PhotoCraft:** Logo oficial em alta resolução integrado nativamente para todas as densidades de tela e launchers.
* 🎮 **Assistente Inicial de Boas-Vindas (Onboarding):** Interface inicial intuitiva (inspirada em emuladores como Yuzu e Skyline) para introdução de recursos e concessão de permissões de armazenamento antes de carregar a tela de trabalho.
* 📁 **Acesso ao Armazenamento Completo:** Suporte a abertura de arquivos PSD, imagens (PNG, JPEG, WebP, TIFF, EXR) e bibliotecas de pincéis (`.abr`) salvos no armazenamento interno ou cartão SD.
* 🖥️ **Samsung DeX & Googlebook OS / Chromebook Ready:**
  * Atividade totalmente redimensionável (`android:resizeableActivity="true"`).
  * Suporte nativo a GPUs Intel UHD (Chromebook Jinlon e derivados) com pipeline dual Vulkan + OpenGL ES (GLES via EGL).
  * Suporte a múltiplas janelas, rotação livre e teclado/mouse/touchpad físicos.
  * Suporte a arrastar e soltar (Drag-and-Drop) de arquivos direto para a janela de edição.
* 📱 **Controles Touch e Escala Dinâmica:**
  * Ajuste rápido de zoom/DPI da interface diretamente na barra superior para telas compactas de celular (evitando cortes e vazamentos de painéis).
  * Rolagem fluida por toque em listas e menus extensos.
* ⚡ **Performance Nativa em Rust (Dual-ABI):** Compilado diretamente para binários de máquina nativos `ARM64 (aarch64-linux-android)` para celulares/tablets e `x86_64 (x86_64-linux-android)` para Chromebooks / Googlebook OS, com aceleração gráfica direta na GPU via `wgpu`.

---

## 🏷️ Versionamento de Releases

Para garantir transparência entre as novidades do aplicativo e as atualizações do motor original:

* **Versão Android:** `v0.1.4-android` (Build 6)
* **Base Upstream:** `PhotoCraft v0.17.0` *(sincronizado continuamente com os lançamentos mais recentes do repositório oficial)*

---

## 📥 Download e Instalação do APK

Você pode baixar o instalador oficial diretamente na aba de **[Releases](https://github.com/nepotira/PhotoCraft-Android/releases)** ou nos artefatos do **[GitHub Actions](https://github.com/nepotira/PhotoCraft-Android/actions)**.

### Passo a passo para instalar:
1. Baixe o arquivo `PhotoCraft-Android.apk` no seu dispositivo Android.
2. Abra o arquivo baixado e selecione **Instalar**. *(Caso o Android solicite, autorize a instalação de apps de fontes desconhecidas para o seu navegador ou gerenciador de arquivos).*
3. Ao abrir o aplicativo pela primeira vez, siga o assistente de boas-vindas:
   * Conceda a permissão de armazenamento para abrir e salvar imagens.
   * Ajuste a escala de interface ideal para o tamanho da sua tela.
4. Pronto! O PhotoCraft estará pronto para criar e editar seus projetos.

---

## 📜 Licença e Atribuição

O PhotoCraft é distribuído sob as licenças **MIT** ou **Apache-2.0**.
* **Projeto Base e Motor Criativo:** Desenvolvido pela equipe [storytold](https://github.com/storytold) e Brandon Thomas ([getartcraft.com](https://getartcraft.com)).
* **Port, Empacotamento e Ajustes para Android / Samsung DeX:** Mantido por **Nepo** ([@nepotira](https://github.com/nepotira)).
