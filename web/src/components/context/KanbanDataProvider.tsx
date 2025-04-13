// web/src/components/context/KanbanDataProvider.tsx
import React, {createContext, useContext, useState, useEffect, useCallback} from 'react'
import {WasmKanbanBoard} from 'bindgen/kandown_wasm'
import {KanbanViewData} from 'bindgen/kandown'

interface KanbanDataContextType {
    board: WasmKanbanBoard | null
    currentView: string
    viewsList: string[]
    boardData: KanbanViewData | null
    loading: boolean
    markdown: string
    setCurrentView: (view: string) => void
    moveCard: (cardId: string, sourceColumn: string, destColumn: string) => void
    addCard: (title: string, description: string, column: string) => boolean
}

const KanbanDataContext = createContext<KanbanDataContextType | null>(null)

export const useKanbanData = () => {
    const context = useContext(KanbanDataContext)
    if (!context) {
        throw new Error('useKanbanData must be used within a KanbanDataProvider')
    }
    return context
}

interface KanbanDataProviderProps {
    board: WasmKanbanBoard
    children: React.ReactNode
}

export function KanbanDataProvider({board, children}: KanbanDataProviderProps) {
    const [viewsList, setViewsList] = useState<string[]>([])
    const [currentView, setCurrentView] = useState<string>('')
    const [boardData, setBoardData] = useState<KanbanViewData | null>(null)
    const [loading, setLoading] = useState(true)
    const [markdown, setMarkdown] = useState('')

    // Initialize data when the board changes
    useEffect(() => {
        if (!board) return

        try {
            console.debug('[KanbanDataProvider] Initializing with board:', board)
            // Get available views
            const views = board.getViewNames() as unknown as string[]
            console.debug('[KanbanDataProvider] Available views:', views)
            setViewsList(views)

            if (views.length > 0) {
                setCurrentView(views[0])
            }

            // Get initial markdown representation
            setMarkdown(board.getMarkdown())
            setLoading(false)
        } catch (error) {
            console.error('[KanbanDataProvider] Error initializing board:', error)
            setLoading(false)
        }
    }, [board])

    // Refresh board data
    const refreshData = useCallback(() => {
        if (!board || !currentView) return

        try {
            console.debug('[KanbanDataProvider] Refreshing data for view:', currentView)
            // Get comprehensive view data
            const viewDataJson = board.getViewData(currentView)
            const viewData = JSON.parse(viewDataJson)
            console.debug('[KanbanDataProvider] Received view data:', viewData)

            // Convert the columns data to the items format needed for the DnD board
            // THIS IS THE FIX - We need to build the items map from the columns data
            const items: Record<string, string[]> = {}

            if (viewData.columns) {
                viewData.columns.forEach((column: any) => {
                    // Extract just the IDs from each card in the column
                    const cardIds = column.cards.map((card: any) =>
                        typeof card === 'string' ? card : card.id,
                    )
                    items[column.id] = cardIds
                })

                console.debug('[KanbanDataProvider] Generated items map:', items)
                viewData.items = items
            } else {
                console.error('[KanbanDataProvider] No columns found in view data')
            }

            // Set the board data (now contains all needed information)
            setBoardData(viewData)

            // Update markdown
            setMarkdown(board.getMarkdown())
        } catch (error) {
            console.error('[KanbanDataProvider] Error loading board data:', error)
        }
    }, [board, currentView])

    // Handle moving a card
    const moveCard = useCallback(
        (cardId: string, sourceColumn: string, destColumn: string) => {
            if (!board || !currentView) return

            try {
                console.debug(
                    `[KanbanDataProvider] Moving card ${cardId} from ${sourceColumn} to ${destColumn}`,
                )
                board.moveCard(cardId, sourceColumn, destColumn, currentView)
                refreshData()
            } catch (error) {
                console.error('[KanbanDataProvider] Error moving card:', error)
            }
        },
        [board, currentView, refreshData],
    )

    // Handle adding a card
    const addCard = useCallback(
        (title: string, description: string, column: string) => {
            if (!board) return false

            try {
                console.debug(
                    `[KanbanDataProvider] Adding card "${title}" to column ${column}`,
                )
                // Create properties JSON with the column as the group property value
                const groupProperty = boardData?.groupByProperty || ''
                const properties: Record<string, string> = {}

                if (groupProperty) {
                    properties[groupProperty] = column
                }

                const propertiesJson = JSON.stringify(properties)
                console.debug(`[KanbanDataProvider] Card properties: ${propertiesJson}`)

                board.addCard(title, description, propertiesJson)
                refreshData()
                return true
            } catch (error) {
                console.error('[KanbanDataProvider] Error adding card:', error)
                return false
            }
        },
        [board, boardData, refreshData],
    )

    // Load data for the current view
    useEffect(() => {
        if (!board || !currentView) return

        refreshData()
    }, [board, currentView, refreshData])

    const value = {
        board,
        currentView,
        viewsList,
        boardData,
        loading,
        markdown,
        setCurrentView,
        moveCard,
        addCard,
        refreshData,
    }

    return <KanbanDataContext.Provider value={value}>{children}</KanbanDataContext.Provider>
}
