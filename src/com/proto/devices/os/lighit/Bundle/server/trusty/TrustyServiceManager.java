// src/com/proto/devices/os/lighit/Bundle/server/trusty/TrustyServiceManager.java
package com.proto.devices.os.lighit.Bundle.server.trusty;

import java.nio.ByteBuffer;
import java.util.UUID;

/**
 * 🔐 Gerenciador de Serviços Confiáveis (Trusty Service Manager).
 * * * Esta classe atua como uma ponte entre as aplicações do LightOS Userspace
 * * e o ambiente de execução confiável (TEE/Trusty), geralmente implementado
 * * através de chamadas de sistema (syscalls) para o Kernel Rust.
 */
public class TrustyServiceManager {

    /**
     * ID de Serviço (UUID) de um serviço confiável no ambiente Trusty TEE.
     * * Exemplo: Gerenciador de Chaves ou Autenticação Biométrica.
     */
    private final UUID serviceId;

    // ------------------------------------------------------------------------
    // --- Métodos de Interação de Baixo Nível (Nativas/FFI) ---
    // ------------------------------------------------------------------------

    // Função nativa para conectar a um serviço Trusty (fazendo uma Syscall para o Kernel Rust)
    private native long nativeConnect(byte[] serviceIdBytes);

    // Função nativa para enviar um comando e receber uma resposta
    private native int nativeSendCommand(long handle, ByteBuffer command, ByteBuffer response);

    // Função nativa para fechar a conexão
    private native void nativeClose(long handle);
    
    // Carregar a biblioteca nativa do Userspace do LightOS que lida com as Syscalls TEE
    static {
        try {
            // Em um Userspace real, esta seria a biblioteca de Syscalls do LightOS
            System.loadLibrary("lightos_trusty_syscalls"); 
        } catch (UnsatisfiedLinkError e) {
            System.err.println("ERRO TRUSTY: Falha ao carregar a biblioteca nativa Trusty.");
        }
    }

    // ------------------------------------------------------------------------
    // --- API Pública do Serviço ---
    // ------------------------------------------------------------------------

    /**
     * @brief Construtor que define o ID do Serviço Confiável.
     * @param serviceId O UUID do serviço TEE alvo (ex: Trusted Keymaster).
     */
    public TrustyServiceManager(UUID serviceId) {
        this.serviceId = serviceId;
        System.out.println("Trusty Manager criado para o serviço: " + serviceId.toString());
    }

    /**
     * @brief Abre uma conexão com o Serviço Confiável.
     * @return Um Handle (identificador) de conexão de 64 bits.
     * @throws RuntimeException se a conexão falhar.
     */
    public long connect() throws RuntimeException {
        // Converte o UUID para um array de bytes para FFI
        ByteBuffer bb = ByteBuffer.wrap(new byte[16]);
        bb.putLong(serviceId.getMostSignificantBits());
        bb.putLong(serviceId.getLeastSignificantBits());
        
        long handle = nativeConnect(bb.array());
        
        if (handle <= 0) {
            throw new RuntimeException("Falha ao conectar ao serviço Trusty. Handle retornado: " + handle);
        }
        return handle;
    }

    /**
     * @brief Envia um comando assíncrono para o serviço confiável.
     * @param handle O Handle de conexão.
     * @param command O buffer de comando a ser enviado.
     * @param response O buffer para receber a resposta do TEE.
     * @return O tamanho da resposta recebida.
     */
    public int sendCommand(long handle, ByteBuffer command, ByteBuffer response) {
        if (handle <= 0) {
            System.err.println("ERRO: Handle de conexão inválido.");
            return -1;
        }
        
        command.flip(); // Prepara o buffer para leitura nativa
        int result = nativeSendCommand(handle, command, response);
        response.flip(); // Prepara o buffer para leitura Java
        
        return result;
    }

    /**
     * @brief Fecha a conexão com o Serviço Confiável.
     * @param handle O Handle de conexão.
     */
    public void close(long handle) {
        if (handle > 0) {
            nativeClose(handle);
            System.out.println("Conexão Trusty fechada (Handle: " + handle + ")");
        }
    }
}
