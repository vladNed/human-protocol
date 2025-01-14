import { configureStore } from '@reduxjs/toolkit';
import { useDispatch } from 'react-redux';

import escrow from './escrow/reducer';
import leader from './leader/reducer';
import token from './token/reducer';

export const store = configureStore({
  reducer: {
    escrow,
    token,
    leader,
  },
  middleware: (getDefaultMiddleware) => getDefaultMiddleware({ thunk: true }),
});

export type AppState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
export const useAppDispatch = () => useDispatch<AppDispatch>();
