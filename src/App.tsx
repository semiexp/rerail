import { foo } from '../rerail-internal/pkg/rerail_internal'

function App() {
    const val = foo(10);
    return (<div>
        {val}
    </div>);
}

export default App;
