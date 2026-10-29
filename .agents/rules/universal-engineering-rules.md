# 🛡️ Regras Universais de Engenharia de Documentação e Versionamento Semântico
# Repositório: PhotoCraft Android (nepotira/PhotoCraft-Android)
# Padrão: Antigravity Universal Engineering Standard (AUES v1.0)

---

## 1. Taxonomia e Estrutura Universal de Documentação

Todo agente de IA (Antigravity ou outro) e desenvolvedor humano que operar neste repositório DEVE manter estritamente a seguinte taxonomia de arquivos:

| Arquivo / Diretório | Função Primária | Regra de Manutenção |
|---|---|---|
| `AGENT_INSTRUCTIONS.md` | Diretrizes permanentes de arquitetura e sobrevivência mobile | NUNCA remover; atualizar sempre que novas pontes JNI ou rotinas de CI forem consolidadas. |
| `AGENTS.md` | Guia de sobrevivência e conformidade com o PhotoCraft upstream | Ler antes de mexer no motor gráfico e nas crates compartilhadas. |
| `VERSIONING.md` | Especificação formal de SemVer 2.0.0, `version_code` e chaves | Consultar OBRIGATORIAMENTE antes de commits de release. |
| `CHANGELOG.md` | Diário de bordo atômico de modificações e correções de bugs | Cada correção DEVE receber tags `[BUG-XXX]` e `[FIX-XXX]` detalhadas. |
| `README.md` | Vitrine pública, badges de build, notas de release e instruções de instalação | Sincronizar links de download, versão corrente e atribuições a cada release. |
| `.agents/rules/` | Regras ativas carregadas automaticamente pelo Antigravity | Preservar as regras universais para garantir paridade em qualquer sessão de chat. |
| `apps/photocraft-android/` | Célula isolada de empacotamento, JNI e runner Android | Todo código específico do sistema Android DEVE permanecer exclusivamente aqui. |
| `crates/ui-egui/` | Casca de interface gráfica adaptada cirurgicamente para touch | Somente intervenções mínimas e reversíveis para multitoque e responsividade. |

---

## 2. Versionamento Semântico Universal (SemVer 2.0 + Android)

### 2.1 Esquema de Duplo Versionamento
Todo lançamento possui duas chaves de versão obrigatórias e sincronizadas:

1. **Tag Semântica Git & Cargo**: `v<MAJOR>.<MINOR>.<PATCH>-android`
   - **MAJOR**: Quebra de compatibilidade grave (ex: mudança arquitetural de engine gráfica, perda de retrocompatibilidade com versões de Android ou reescrita total).
   - **MINOR**: Novas funcionalidades mobile (ex: novo módulo de exportação, integração com canetas ativas S-Pen/USI, suporte expandido ao DeX, sincronização upstream).
   - **PATCH**: Correções de bugs, ajustes de escala de DPI, correções de layout responsivo, hotfixes de permissões JNI.
   - **Sufixo `-android`**: Identificador obrigatório do fork mobile.

2. **Android `version_code` / Build Number (Monotônico Crescente)**:
   - Inteiro estritamente crescente (+1 a cada compilação de release).
   - NUNCA repetir ou decrementar o número de build, sob pena de bloqueio pelo `PackageInstaller` do Android (`INSTALL_FAILED_VERSION_DOWNGRADE`).

3. **Rastreamento de Upstream**:
   - Todo release e entrada de changelog DEVE declarar explicitamente a versão base do PhotoCraft original da storytold:
     ```markdown
     *Base Upstream: PhotoCraft vX.Y.Z*
     ```

---

## 3. Integridade Criptográfica e Assinatura de Pacotes

1. **Keystore Permanente**:
   - O arquivo `apps/photocraft-android/release.keystore` é a identidade criptográfica oficial do aplicativo.
   - **É EXPRESSAMENTE PROIBIDO** deletar, substituir ou gerar novas keystores em tempo de CI.
   - Parâmetros imutáveis:
     - Alias: `androidkey`
     - Senha da Keystore: `android`
     - Senha da Chave: `android`
     - Algoritmo: RSA 2048 bits

2. **Compatibilidade de Atualização**:
   - Manter a mesma chave garante que o usuário consiga atualizar o APK baixado sem desinstalar a versão anterior, preservando seus documentos, pincéis e configurações.

---

## 4. Política "Never Crash" e Robustez em Rust (Rust 2024)

1. **Zero Panic em Código de Produção**:
   - Código que não seja de teste NUNCA deve chamar `unwrap()`, `expect()`, `panic!()` ou `todo!()`.
   - Utilize sempre propagação de erros via `Result<T, E>` e operador `?`.
   - Trate ponteiros JNI nulos e chamadas falhas graciosamente com fallbacks silenciosos ou logs descritivos via `log::warn!` / `log::error!`.

2. **Defesa Contra Entradas Hostis**:
   - Toques múltiplos na tela e dados de zoom/pinça devem ser filtrados com limites de sanidade (`clamp`) e verificação de finitude (`is_finite()`).
   - Bloquear criação de traços de pincel quando houver 2 ou mais dedos em contato com a tela.

3. **Conformidade de Formatação**:
   - Todo código Rust modificado DEVE respeitar o `rustfmt.toml` do repositório (`max_width = 160`, `use_small_heuristics = "Max"`).
   - Sempre valide com `cargo fmt --check` para evitar quebra no workflow de CI `ci.yml`.

---

## 5. Regras de Isolamento Mobile e Merge Upstream

1. **Princípio da Não-Poluição**:
   - Nenhuma crate do motor básico (`crates/engine`, `crates/doc`, `crates/codecs`, `crates/io`, `crates/geom`, etc.) deve conter dependências diretas de Android (`jni`, `android_logger`, `cargo-apk`).
   - O downstream Android deve consumir o motor como dependência limpa.

2. **Fluxo de Merge com o Upstream**:
   - Para sincronizar com novas versões do `storytold/photocraft`:
     ```bash
     git fetch upstream main
     git merge upstream/main -m "chore(upstream): sync latest PhotoCraft updates"
     ```
   - Em caso de conflito nos arquivos `crates/ui-egui/src/canvas.rs` ou `crates/ui-egui/src/dialogs.rs`, PRESERVE SEMPRE os blocos de multitoque e de responsividade.

---

## 6. Protocolo de Registro de Erros e Transferência (Handoff)

Sempre que um erro for identificado pelo usuário ou nos pipelines:

1. Registre o bug na documentação com a estrutura:
   - `[BUG-XXX] Nome Descritivo`:
     - **Sintoma**: Como o erro se manifesta para o usuário.
     - **Causa Raiz Técnica**: Qual arquivo, linha, chamada de API ou condição de corrida causou a falha.
     - **Status / Solução**: `[PENDENTE]` com proposta de ação ou `[FIX-XXX]` com a resolução implementada.
2. Atualize o `CHANGELOG.md` na seção correspondente.
3. Garanta que o estado do repositório seja comitado com mensagens semânticas claras (`fix:`, `feat:`, `chore:`, `release:`).

---

## 7. Política de Higienização de Artefatos Locais (Mandato Permanente)

- **Regra de Ouro**: Sempre que uma nova compilação for gerada e os novos APKs forem transferidos/baixados para a pasta local `dist/`, **TODOS os APKs de versões anteriores DEVEM ser sumariamente excluídos do computador/armazenamento local**.
- Apenas os binários correspondentes à versão corrente mais recente devem permanecer no diretório local de distribuição (`dist/`), evitando acúmulo de arquivos obsoletos e confusão na instalação manual.

---

## 8. Arquitetura de Plataforma de Destino: Android Mobile & GooglebookOS

- **GooglebookOS é Bare-Metal (NÃO é Container)**: O GooglebookOS (como no hardware Jinlon / Intel Comet Lake) é um **sistema operacional nativo bare-metal para laptops e PCs** rodando Android diretamente sobre o hardware x86_64, e **NÃO** um container ou máquina virtual (ARCVM) emulado dentro do ChromeOS.
- **Aceleração Gráfica Direta**: A renderização gráfica conversa diretamente com o driver Mesa e a GPU Intel UHD (`/dev/dri/renderD128`) via pipeline dual WGPU (`Backends::VULKAN | Backends::GL`).
- **Janelamento Desktop Completo**: O aplicativo é uma janela desktop de primeira classe no GooglebookOS, com redimensionamento livre, atalhos de teclado do Photoshop e ponteiros de mouse nativos.


