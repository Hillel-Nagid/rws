import axios from "axios";


export function useBucket() {
    const token = localStorage.getItem('token');
    const axiosClient = axios.create({
        headers: {
          Authorization: `Bearer ${token}`, //todo 
        },
    });

    function getAllBuckets(): any {
        axios.
    }


}