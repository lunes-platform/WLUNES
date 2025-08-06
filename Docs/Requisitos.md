# Documento de Requisitos: Contrato Wrapped Lunes (WLUNES)

**Versão: 1.0**  
**Data: 05/08/2025**  
**Autor: Lunex Team**  
**Rede Alvo Test: Lunes Network (`wss://ws-test.lunes.io`)**
**Rede Alvo: Lunes Network (`wss://ws.lunes.io`)**
**Ink! Version: 5.1.1**
https://github.com/lunes-platform/lunes-nightly 
https://github.com/lunes-platform/Lunex 


---
**Não usar OpenBrush está descontinuar.**

**Usar padão nativo do Ink! para implementar o contrato. Analise adocumentação do Ink! para entender como implementar o contrato.**


## 1. Visão Geral e Propósito

O ativo nativo da Lunes Network, **$LUNES**, não segue nativamente o padrão de token `PSP22`. A Lunex DEX, por sua vez, foi projetada para operar com contratos que aderem a este padrão, a fim de garantir interações seguras e padronizadas.

O contrato **Wrapped Lunes (WLUNES)** servirá como uma ponte essencial para resolver essa incompatibilidade. Seu único propósito é "embrulhar" a moeda nativa `$LUNES` em um contrato `PSP22` totalmente compatível. Este contrato não terá funcionalidades complexas, governança ou chaves de administrador; seu funcionamento deve ser 100% autônomo, transparente e seguro.

A principal invariante do sistema é que a quantidade de `WLUNES` em circulação (`total_supply`) deve ser **exatamente igual** à quantidade de `$LUNES` nativos bloqueados no saldo do contrato.

## 2. Requisitos Funcionais Core

O contrato deve prover duas funções principais para o processo de wrap/unwrap.

### 2.1. `deposit()` - Embrulhar LUNES em WLUNES
-   **Descrição:** Permite que qualquer usuário converta `$LUNES` nativos em tokens `WLUNES` na proporção de 1:1.
-   **Mecanismo:** A função deve ser `payable` (`#[ink(message, payable)]`).
-   **Lógica:**
    1.  Receber os `$LUNES` enviados na transação através de `self.env().transferred_value()`.
    2.  Validar que o valor recebido é maior que zero.
    3.  Cunhar (mint) uma quantidade idêntica de tokens `WLUNES` para o endereço do chamador (`self.env().caller()`).
    4.  Emitir um evento `Transfer` correspondente à cunhagem (de `ZERO_ADDRESS` para o chamador).

### 2.2. `withdraw(amount: Balance)` - Desembrulhar WLUNES para LUNES
-   **Descrição:** Permite que qualquer usuário que possua `WLUNES` os converta de volta para a moeda nativa `$LUNES` na proporção de 1:1.
-   **Mecanismo:** A função deve receber a quantidade de `WLUNES` a ser desembrulhada como um parâmetro.
-   **Lógica:**
    1.  Validar que o `amount` a ser retirado é maior que zero.
    2.  Verificar se o chamador (`self.env().caller()`) possui saldo de `WLUNES` suficiente.
    3.  Queimar (burn) a quantidade (`amount`) de `WLUNES` do saldo do chamador.
    4.  Transferir a mesma quantidade (`amount`) de `$LUNES` nativos do saldo do contrato para o endereço do chamador.
    5.  Emitir um evento `Transfer` correspondente à queima (do chamador para `ZERO_ADDRESS`).

## 3. Conformidade com o Padrão PSP22

O contrato **deve** implementar integralmente a interface `PSP22`, incluindo as extensões `PSP22Metadata` e `PSP22Burnable`.

### 3.1. PSP22 Core (`#[p sp22]`)
-   `total_supply()`: Deve retornar o total de `WLUNES` em circulação.
-   `balance_of(owner)`: Deve retornar o saldo de `WLUNES` de um endereço específico.
-   `transfer(to, value, data)`: Transferência padrão de `WLUNES`.
-   `transfer_from(from, to, value, data)`: Transferência baseada em `allowance`.
-   `approve(spender, value)`: Definir permissão de gasto.
-   `allowance(owner, spender)`: Consultar permissão de gasto.

### 3.2. PSP22 Metadata (`#[psp22_metadata]`)
-   `token_name()`: Deve retornar a `String` `"Wrapped Lunes"`.
-   `token_symbol()`: Deve retornar a `String` `"WLUNES"`.
-   `token_decimals()`: Deve retornar `8`, para manter a consistência com a precisão da moeda nativa LUNES.

### 3.3. PSP22 Burnable (`#[psp22_burnable]`)
-   `burn(from, value)`: A função `withdraw` deve usar internamente esta lógica para queimar os tokens.

## 4. Requisitos de Segurança e Invariantes

A segurança é o pilar deste contrato, pois ele guardará fundos nativos da rede.

-   **Relação 1:1 (Invariante Crítica):** O contrato deve garantir matematicamente que, em nenhum momento, `self.env().balance()` (saldo de LUNES nativos) seja menor que `total_supply()` (total de WLUNES).
-   **Proteção contra Overflow/Underflow:** Todas as operações aritméticas, especialmente as de `mint` e `burn`, devem obrigatoriamente usar métodos seguros como `checked_add` e `checked_sub` para prevenir falhas.
-   **Sem Funções de Admin:** O contrato não deve ter um `owner` ou qualquer função privilegiada que possa cunhar, queimar, pausar ou confiscar fundos. Deve ser 100% autônomo e imutável após o deploy.
-   **Proteção contra Reentrância:** Embora o fluxo simples de `deposit/withdraw` minimize o risco, a implementação deve seguir o padrão *Checks-Effects-Interactions*. A função `withdraw`, por exemplo, deve primeiro queimar os tokens `WLUNES` do usuário (Effect) e só então transferir os `$LUNES` nativos (Interaction).

## 5. Eventos

O contrato deve emitir os eventos padrão da interface `PSP22`:

-   **`Transfer`:**
    -   Na função `deposit`: `from: ZERO_ADDRESS`, `to: caller`.
    -   Na função `withdraw`: `from: caller`, `to: ZERO_ADDRESS`.
    -   Em transferências normais: `from: sender`, `to: recipient`.
-   **`Approval`:**
    -   Na função `approve`.

## 6. Interface Pública do Contrato (ABI Resumida)

A seguir, a lista de todas as mensagens públicas que o contrato `WLUNES` deve expor:

| Função (Mensagem)                      | Tipo      | `Payable` | Descrição                                               |
| -------------------------------------- | --------- | :-------: | ------------------------------------------------------- |
| `new()`                                | Construtor|    Não    | Cria o contrato (sem parâmetros).                       |
| `deposit()`                            | `&mut`    |   **Sim** | Converte `$LUNES` nativos em `WLUNES`.                  |
| `withdraw(amount: Balance)`            | `&mut`    |    Não    | Converte `WLUNES` de volta para `$LUNES` nativos.       |
| `psp22::*` (todas as funções do Padrão) | `&mut`/`&`|    Não    | Funções padrão para um token `PSP22`.                   |
| `psp22_metadata::*`                    | `&`       |    Não    | Funções para retornar nome, símbolo e decimais.         |


## 7. Considerações de Implantação

-   O construtor do contrato não deve receber nenhum parâmetro.
-   Uma vez implantado na rede Lunes, o endereço do contrato `WLUNES` será considerado canônico e será configurado no construtor do `Router` da Lunex DEX.
-   O contrato não deve ser atualizável (`set_code_hash` deve ser desabilitado ou inacessível) para garantir a confiança dos usuários de que as regras nunca mudarão.

---

## 8. Exemplo de Implementação (Esqueleto do Código)

O código a seguir serve como um esqueleto e ponto de partida para o desenvolvimento, ilustrando a estrutura principal do contrato com base nos requisitos acima. A lógica interna das funções `PSP22` pode ser implementada utilizando a biblioteca `INK! 5.1.1` para acelerar o desenvolvimento e aumentar a segurança.


# Requisitos para o Contrato WLUNES (Wrapped LUNES)

**Versão 1.0.0**  
**Ink! Version:** 5.1.1  
**Rede Alvo:** Lunes Network (`wss://ws.lunes.io`)  
**Última Atualização:** Agosto 2024

Este documento define os requisitos técnicos e funcionais para o desenvolvimento do contrato `WLUNES` (Wrapped LUNES), que é essencial para permitir que a moeda nativa LUNES seja negociada na Lunex DEX.

**📋 Especificações Técnicas:**
- **Framework:** ink! 5.1.1 (Polkadot Smart Contracts)
- **Padrão de Token:** PSP22 (Polkadot Standard Proposal)
- **Relação:** 1:1 com LUNES nativo
- **Decimais:** 8 (consistente com a precisão da moeda nativa)

---

## 🎯 Objetivo

O contrato `WLUNES` serve como um "adaptador" que converte a moeda nativa LUNES em um token PSP22 compatível, permitindo que ela seja negociada na DEX da mesma forma que qualquer outro token PSP22.

---

## 📋 Requisitos Funcionais

### **1. Função `deposit` (payable)**
- **Descrição:** Converte LUNES nativos em WLUNES (1:1)
- **Parâmetros:** Nenhum (usa `transferred_value()`)
- **Validações:**
  - Verificar se `transferred_value() > 0`
  - Implementar checked arithmetic para evitar overflow
- **Ações:**
  - Mint WLUNES para o `caller()` na quantidade de LUNES enviados
  - Emitir evento `Transfer` (de ZERO_ADDRESS para caller)
- **Retorno:** `Result<(), PSP22Error>`

### **2. Função `withdraw`**
- **Descrição:** Converte WLUNES de volta para LUNES nativos (1:1)
- **Parâmetros:** `amount: Balance`
- **Validações:**
  - Verificar se `amount > 0`
  - Verificar se caller tem saldo suficiente
  - Implementar checked arithmetic
- **Ações:**
  - Burn WLUNES do caller
  - Transferir LUNES nativos do contrato para o caller
  - Emitir evento `Transfer` (de caller para ZERO_ADDRESS)
- **Retorno:** `Result<(), PSP22Error>`

---

## 🔒 Requisitos de Segurança

### **1. Invariantes de Segurança**
- **Relação 1:1:** Sempre deve haver 1 WLUNES para cada LUNES nativo no contrato
- **Sem Admin Functions:** O contrato não deve ter funções administrativas
- **Reentrancy Protection:** Implementar guards para prevenir ataques de reentrância
- **Checked Arithmetic:** Todas as operações matemáticas devem usar `checked_*` methods

### **2. Validações Obrigatórias**
- **Zero Amount:** Rejeitar operações com valor zero
- **Zero Address:** Validar endereços antes de operações
- **Balance Checks:** Verificar saldos antes de operações
- **Overflow Protection:** Usar `checked_add`, `checked_sub`, etc.

### **3. Padrão Checks-Effects-Interactions**
1. **Checks:** Validar todas as condições
2. **Effects:** Atualizar estado interno (mint/burn)
3. **Interactions:** Transferir tokens nativos (se aplicável)

---

## 📊 Interface PSP22

### **Core PSP22 (Obrigatório)**
- `transfer(to: AccountId, value: Balance) -> Result<(), PSP22Error>`
- `transfer_from(from: AccountId, to: AccountId, value: Balance) -> Result<(), PSP22Error>`
- `approve(spender: AccountId, value: Balance) -> Result<(), PSP22Error>`
- `balance_of(owner: AccountId) -> Balance`
- `allowance(owner: AccountId, spender: AccountId) -> Balance`
- `total_supply() -> Balance`

### **Metadata PSP22 (Obrigatório)**
- `token_name() -> Option<String>` → `"Wrapped Lunes"`
- `token_symbol() -> Option<String>` → `"WLUNES"`
- `token_decimals() -> u8` → `8`

### **Burnable PSP22 (Opcional, mas Recomendado)**
- `burn(value: Balance) -> Result<(), PSP22Error>`
- `burn_from(from: AccountId, value: Balance) -> Result<(), PSP22Error>`

---

## 📝 Eventos

### **Transfer Event (Obrigatório)**
```rust
#[ink(event)]
pub struct Transfer {
    #[ink(topic)]
    from: Option<AccountId>,
    #[ink(topic)]
    to: Option<AccountId>,
    value: Balance,
}
```

### **Approval Event (Obrigatório)**
```rust
#[ink(event)]
pub struct Approval {
    #[ink(topic)]
    owner: AccountId,
    #[ink(topic)]
    spender: AccountId,
    value: Balance,
}
```

---

## 🔧 Interface Pública

### **Funções Específicas do WLUNES**
- `deposit() -> Result<(), PSP22Error>` (payable)
- `withdraw(amount: Balance) -> Result<(), PSP22Error>`

### **Funções PSP22 Padrão**
- Todas as funções do padrão PSP22 conforme especificado acima

---

## 🧪 Testes Obrigatórios

### **1. Testes de Funcionalidade**
- `test_deposit_success()`: Depositar LUNES e verificar mint de WLUNES
- `test_withdraw_success()`: Queimar WLUNES e verificar transferência de LUNES
- `test_deposit_zero_amount()`: Rejeitar depósito de valor zero
- `test_withdraw_zero_amount()`: Rejeitar saque de valor zero
- `test_withdraw_insufficient_balance()`: Rejeitar saque sem saldo suficiente

### **2. Testes de Segurança**
- `test_reentrancy_protection()`: Verificar proteção contra reentrância
- `test_overflow_protection()`: Verificar proteção contra overflow
- `test_1_1_ratio_maintained()`: Verificar que a relação 1:1 é mantida

### **3. Testes de Integração**
- `test_psp22_compliance()`: Verificar conformidade com PSP22
- `test_metadata_correct()`: Verificar metadados corretos
- `test_events_emitted()`: Verificar emissão de eventos

---

## 📦 Exemplo de Esqueleto de Código (Rust/ink!)

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
#[derive(Default)]
#[ink(storage)]
pub struct Wlunes {
    #[ink(embed)]
    psp22: psp22::Data,
    // Não há campos de admin, owner, etc.
}

impl psp22::PSP22 for Wlunes {}
impl psp22::extensions::metadata::PSP22Metadata for Wlunes {}
impl psp22::extensions::burnable::PSP22Burnable for Wlunes {}

impl Wlunes {
    #[ink(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Converte LUNES nativos em WLUNES (1:1)
    #[ink(message, payable)]
    pub fn deposit(&mut self) -> Result<(), PSP22Error> {
        let amount = self.env().transferred_value();
        let caller = self.env().caller();

        if amount == 0 {
            return Err(PSP22Error::Custom(String::from("ZeroAmount")));
        }

        // Mint WLUNES para o chamador
        self.psp22._mint_to(caller, amount)?;

        // Emitir evento Transfer (de ZERO_ADDRESS para caller)
        self.env().emit_event(psp22::Transfer {
            from: Some(AccountId::from([0u8; 32])),
            to: Some(caller),
            value: amount,
        });

        Ok(())
    }

    /// Converte WLUNES de volta para LUNES nativos (1:1)
    #[ink(message)]
    pub fn withdraw(&mut self, amount: Balance) -> Result<(), PSP22Error> {
        let caller = self.env().caller();

        if amount == 0 {
            return Err(PSP22Error::Custom(String::from("ZeroAmount")));
        }

        // 1. Queimar WLUNES do chamador (Effect)
        self.psp22._burn_from(caller, amount)?;

        // 2. Transferir LUNES nativos do contrato para o chamador (Interaction)
        self.env().transfer(caller, amount)
            .map_err(|_| PSP22Error::Custom(String::from("NativeTransferFailed")))?;

        // Emitir evento Transfer (de caller para ZERO_ADDRESS)
        self.env().emit_event(psp22::Transfer {
            from: Some(caller),
            to: Some(AccountId::from([0u8; 32])),
            value: amount,
        });

        Ok(())
    }

    // Implementação dos metadados PSP22
    #[ink(message)]
    pub fn token_name(&self) -> Option<String> {
        Some(String::from("Wrapped Lunes"))
    }

    #[ink(message)]
    pub fn token_symbol(&self) -> Option<String> {
        Some(String::from("WLUNES"))
    }

    #[ink(message)]
    pub fn token_decimals(&self) -> u8 {
        8 // Consistente com a precisão da moeda nativa LUNES
    }
}
```

---

## 🚀 Deploy e Integração

### **1. Compilação**
```bash
cargo contract build --manifest-path contracts/wnative/Cargo.toml --release
```

### **2. Deploy**
- Deploy na rede Lunes Network
- Anotar o `AccountId` do contrato deployado
- Verificar o contrato no explorador da rede

### **3. Integração com Lunex DEX**
- O `AccountId` do WLUNES será usado como parâmetro no construtor do `Router`
- O WLUNES será listado automaticamente na DEX via `admin_list_token`

---

## 📚 Referências

- [ink! Documentation](https://use.ink/)
- [PSP22 Standard](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md)
- [Lunes Network Documentation](https://docs.lunes.io/)
- [Lunex DEX Architecture](../LISTING_POLICY.md) 