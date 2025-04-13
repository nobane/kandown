// web/src/components/app/App.tsx

import {useEffect, useState} from 'react'
import {WasmKanbanBoard} from 'bindgen/kandown_wasm'
import {KanbanDataProvider, useKanbanData} from 'src/components/context/KanbanDataProvider'
import {KanbanBoardView} from 'src/components/views/BoardView'
import {TableView} from 'src/components/views/TableView'
import css from './App.module.css'

function AppContent() {
    const {viewsList, currentView, setCurrentView, loading, markdown, boardData} =
        useKanbanData()
    const [activeTab, setActiveTab] = useState<'board' | 'table'>('board')

    useEffect(() => {
        console.debug('[AppContent] Board data:', boardData)
    }, [boardData])

    if (loading) {
        return (
            <div className={css.loading}>
                <div className={css.pulse}></div>
            </div>
        )
    }

    return (
        <div className={css.container}>
            <div className={css.tabs}>
                {/* View selector */}
                <div className={css.viewSelector}>
                    <select
                        value={currentView}
                        onChange={e => setCurrentView(e.target.value)}
                        className={css.viewSelect}
                    >
                        {viewsList.map(view => (
                            <option key={view} value={view}>
                                {view}
                            </option>
                        ))}
                    </select>
                </div>

                {/* Layout tabs */}
                <div className={css.layoutTabs}>
                    <button
                        className={`${css.tabButton} ${
                            activeTab === 'board' ? css.activeTab : ''
                        }`}
                        onClick={() => setActiveTab('board')}
                    >
                        <span className={css.icon}>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                            >
                                <rect width="7" height="7" x="17" y="3" rx="1"></rect>
                                <rect width="7" height="7" x="17" y="14" rx="1"></rect>
                                <rect width="7" height="7" x="3" y="3" rx="1"></rect>
                                <rect width="7" height="11" x="3" y="14" rx="1"></rect>
                            </svg>
                        </span>
                        Board
                    </button>
                    <button
                        className={`${css.tabButton} ${
                            activeTab === 'table' ? css.activeTab : ''
                        }`}
                        onClick={() => setActiveTab('table')}
                    >
                        <span className={css.icon}>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                            >
                                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                                <line x1="3" y1="9" x2="21" y2="9"></line>
                                <line x1="3" y1="15" x2="21" y2="15"></line>
                                <line x1="9" y1="3" x2="9" y2="21"></line>
                                <line x1="15" y1="3" x2="15" y2="21"></line>
                            </svg>
                        </span>
                        Table
                    </button>
                </div>
            </div>

            <div className={css.viewContainer}>
                {activeTab === 'board' && currentView && (
                    <div style={{height: '100%', overflow: 'auto'}}>
                        <KanbanBoardView />
                    </div>
                )}
                {activeTab === 'table' && currentView && <TableView />}
            </div>

            {/* Optional: Display the markdown */}
            <div className={css.markdownPreview}>
                <h3>Markdown Output</h3>
                <pre>{markdown}</pre>
            </div>
        </div>
    )
}

function App(props: {board: () => Promise<WasmKanbanBoard>}) {
    const [board, setBoard] = useState<WasmKanbanBoard | null>(null)

    useEffect(() => {
        async function loadBoard() {
            try {
                console.debug('[App] Loading board...')
                const loadedBoard = await props.board()
                console.debug('[App] Board loaded successfully:', loadedBoard)
                setBoard(loadedBoard)
            } catch (err) {
                console.error('[App] Error loading board:', err)
            }
        }

        loadBoard()
    }, [board, props])

    if (!board) {
        return (
            <div className={css.loading}>
                <div className={css.pulse}></div>
            </div>
        )
    }

    return (
        <KanbanDataProvider board={board}>
            <AppContent />
        </KanbanDataProvider>
    )
}

export default App
