// web/src/utils/helpers.ts
import {UniqueIdentifier} from '@dnd-kit/core'

const defaultInitializer = (index: number) => index

export function createRange<T = number>(
    length: number,
    initializer: (index: number) => any = defaultInitializer,
): T[] {
    return [...new Array(length)].map((_, index) => initializer(index))
}

export function getItemColor(id: UniqueIdentifier) {
    switch (String(id)[0]) {
        case 'A':
            return '#7193f1'
        case 'B':
            return '#ffda6c'
        case 'C':
            return '#00bcd4'
        case 'D':
            return '#ef769f'
        default:
            return undefined
    }
}
