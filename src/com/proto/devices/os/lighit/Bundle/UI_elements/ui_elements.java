// src/com/proto/devices/os/lighit/Bundle/UI_elements/ui_elements.java
package com.proto.devices.os.lighit.Bundle.UI_elements;

/**
 * Representa uma classe base para um Elemento de Interface de Usuário (UI) no LightOS
 * (Assumindo a existência de uma camada de usuário Java/GUI no futuro).
 * * Esta classe encapsula as propriedades básicas de qualquer widget.
 */
public class UI_elements {

    // Identificador único para este elemento na hierarquia de UI
    private final int elementId;
    
    // Posição e dimensões na tela (coordenadas de tela do Kernel/Userspace)
    private int x;
    private int y;
    private int width;
    private int height;
    
    // Texto ou conteúdo exibido
    private String textContent;
    
    // Visibilidade do elemento
    private boolean isVisible;

    /**
     * Construtor para inicializar o elemento de UI.
     * @param elementId ID único.
     * @param x Coordenada X inicial.
     * @param y Coordenada Y inicial.
     * @param textContent Conteúdo de texto inicial.
     */
    public UI_elements(int elementId, int x, int y, String textContent) {
        this.elementId = elementId;
        this.x = x;
        this.y = y;
        this.width = 100; // Valor padrão
        this.height = 30; // Valor padrão
        this.textContent = textContent;
        this.isVisible = true;
    }

    // --- Métodos de Interação e Propriedades ---

    /**
     * Move o elemento para uma nova posição.
     * @param newX Nova coordenada X.
     * @param newY Nova coordenada Y.
     */
    public void moveElement(int newX, int newY) {
        this.x = newX;
        this.y = newY;
        System.out.println("Elemento " + elementId + " movido para (" + newX + ", " + newY + ")");
        // Em um SO real, isso dispararia uma chamada de sistema para o subsistema gráfico do Kernel.
    }
    
    /**
     * Altera o conteúdo de texto.
     * @param newContent Novo conteúdo de texto.
     */
    public void setText(String newContent) {
        this.textContent = newContent;
    }

    // --- Getters ---
    public int getElementId() {
        return elementId;
    }

    public int getX() {
        return x;
    }
    
    public String getTextContent() {
        return textContent;
    }
    
    public boolean isVisible() {
        return isVisible;
    }
}
